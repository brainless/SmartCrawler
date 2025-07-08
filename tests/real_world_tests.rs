use serial_test::serial;
use smart_crawler::{Browser, FetchStatus, HtmlParser, UrlStorage};
use std::collections::{HashMap, HashSet};
use tokio;

/// Full SmartCrawler pipeline that processes a URL using complete functionality
/// including link discovery, root URL prioritization, and domain-level duplicate filtering
async fn full_crawl_pipeline(
    initial_url: &str,
) -> Result<(smart_crawler::HtmlNode, UrlStorage), String> {
    // Initialize crypto provider for rustls (required for HTTPS connections)
    let _ = rustls::crypto::ring::default_provider().install_default();

    let mut browser = Browser::new(4444);
    let parser = HtmlParser::new();
    let mut storage = UrlStorage::new();

    // Extract domain from initial URL
    let domain = smart_crawler::utils::extract_domain_from_url(initial_url)
        .ok_or("Failed to extract domain from URL")?;

    println!("Starting full SmartCrawler pipeline for domain: {}", domain);

    // Connect to WebDriver
    browser
        .connect()
        .await
        .map_err(|e| format!("Failed to connect to WebDriver: {}", e))?;

    // Phase 1: Preparation stage - collect URLs from same domain
    println!("Phase 1: Preparation stage - collecting URLs from same domain");

    let mut domain_urls: HashMap<String, HashSet<String>> = HashMap::new();
    domain_urls
        .entry(domain.clone())
        .or_default()
        .insert(initial_url.to_string());
    storage.add_url(initial_url.to_string());

    // Add root URL for the domain if not already present
    let root_url = smart_crawler::utils::construct_root_url(&domain);
    if !domain_urls[&domain].contains(&root_url) {
        domain_urls
            .get_mut(&domain)
            .unwrap()
            .insert(root_url.clone());
        storage.add_url(root_url.clone());
        println!("Added root URL for domain {}: {}", domain, root_url);
    }

    // For the domain, try to find additional URLs (up to 3 total)
    if domain_urls[&domain].len() < 3 {
        println!(
            "Domain {} has only {} URL(s), searching for more...",
            domain,
            domain_urls[&domain].len()
        );

        // Pick the first URL to extract links from
        if let Some(first_url) = domain_urls[&domain].iter().next() {
            match process_url(&mut browser, &parser, &mut storage, first_url, true).await {
                Ok(html_source) => {
                    let additional_urls = parser.extract_links(&html_source, &domain);
                    let mut added_count = 0;

                    for additional_url in additional_urls {
                        if domain_urls[&domain].len() >= 3 {
                            break;
                        }
                        if domain_urls
                            .get_mut(&domain)
                            .unwrap()
                            .insert(additional_url.clone())
                        {
                            storage.add_url(additional_url);
                            added_count += 1;
                        }
                    }

                    println!(
                        "Found {} additional URLs for domain {}",
                        added_count, domain
                    );
                }
                Err(e) => {
                    println!("Failed to extract links from {}: {}", first_url, e);
                }
            }
        }
    }

    // Phase 2: Process all URLs with root URL prioritization
    println!("Phase 2: Processing all URLs with root URL prioritization");

    let mut all_urls: Vec<String> = Vec::new();

    // First, add the initial user-specified URL
    all_urls.push(initial_url.to_string());

    // Then, add root URL for the domain (if not the user-specified URL)
    if initial_url != root_url {
        all_urls.push(root_url);
    }

    // Finally, add all other discovered URLs
    for url in &domain_urls[&domain] {
        if url != initial_url && !smart_crawler::utils::is_root_url(url) {
            all_urls.push(url.clone());
        }
    }

    // Process all URLs
    for url in &all_urls {
        if let Some(url_data) = storage.get_url_data(url) {
            if matches!(url_data.status, FetchStatus::Success) {
                continue; // Already processed
            }
        }

        match process_url(&mut browser, &parser, &mut storage, url, false).await {
            Ok(_) => println!("Successfully processed {}", url),
            Err(e) => println!("Failed to process {}: {}", url, e),
        }
    }

    // Phase 3: Analyze domain duplicates
    println!("Phase 3: Analyzing domain-level duplicate nodes");

    storage.analyze_domain_duplicates(&domain);
    if let Some(duplicates) = storage.get_domain_duplicates(&domain) {
        let duplicate_count = duplicates.get_duplicate_count();
        if duplicate_count > 0 {
            println!(
                "Found {} duplicate node patterns for domain {}",
                duplicate_count, domain
            );
        } else {
            println!(
                "No duplicate patterns found for domain {} (likely insufficient pages)",
                domain
            );
        }
    }

    // Clean up
    let _ = browser.close().await;

    // Get the processed HTML tree for the initial URL
    let html_tree = storage
        .get_url_data(initial_url)
        .and_then(|data| data.html_tree.as_ref())
        .ok_or("Failed to get HTML tree for initial URL")?
        .clone();

    println!(
        "SmartCrawler pipeline completed successfully for domain: {}",
        domain
    );
    Ok((html_tree, storage))
}

/// Helper function to process a URL (matches main.rs implementation)
async fn process_url(
    browser: &mut Browser,
    parser: &HtmlParser,
    storage: &mut UrlStorage,
    url: &str,
    return_html: bool,
) -> Result<String, String> {
    println!("Processing URL: {}", url);

    if let Some(url_data) = storage.get_url_data_mut(url) {
        url_data.update_status(FetchStatus::InProgress);
    }

    match browser.navigate_to(url).await {
        Ok(()) => match browser.get_html_source().await {
            Ok(html_source) => {
                let title = browser.get_page_title().await.ok();
                let html_tree = parser.parse(&html_source);

                if let Some(url_data) = storage.get_url_data_mut(url) {
                    url_data.set_html_data(html_source.clone(), html_tree, title);
                    url_data.update_status(FetchStatus::Success);
                }

                if return_html {
                    Ok(html_source)
                } else {
                    Ok(String::new())
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to get HTML source: {e}");
                if let Some(url_data) = storage.get_url_data_mut(url) {
                    url_data.update_status(FetchStatus::Failed(error_msg.clone()));
                }
                Err(error_msg)
            }
        },
        Err(e) => {
            let error_msg = format!("Failed to navigate: {e}");
            if let Some(url_data) = storage.get_url_data_mut(url) {
                url_data.update_status(FetchStatus::Failed(error_msg.clone()));
            }
            Err(error_msg)
        }
    }
}

#[tokio::test]
#[ignore] // Ignored by default, run with: cargo test real_world -- --ignored
#[serial]
async fn test_hacker_news_submissions() {
    println!("Testing Hacker News for 30 submission elements using full SmartCrawler pipeline...");

    match full_crawl_pipeline("https://news.ycombinator.com/").await {
        Ok((tree, storage)) => {
            // Apply domain-level duplicate filtering if available
            let filtered_tree = if let Some(domain_duplicates) =
                storage.get_domain_duplicates("news.ycombinator.com")
            {
                println!("Applying domain-level duplicate filtering...");
                smart_crawler::HtmlParser::filter_domain_duplicates(&tree, domain_duplicates)
            } else {
                println!("No domain duplicates found, using original tree");
                tree
            };

            // Find all submission elements using the specified path
            let submissions = filtered_tree.find_by_path(
                "html body center table tbody tr td table tbody tr.athing.submission td.title span.titleline",
            );

            println!(
                "Found {} submission elements after filtering",
                submissions.len()
            );

            // Print details of found submissions for debugging
            for (i, submission) in submissions.iter().enumerate() {
                let content = submission.content.chars().take(50).collect::<String>();
                let status = if content == "[FILTERED DUPLICATE]" {
                    " (FILTERED)"
                } else {
                    ""
                };
                println!("Submission {}: '{}'{}", i + 1, content, status);
            }

            // Check how many are actual content vs filtered duplicates
            let actual_submissions: Vec<_> = submissions
                .iter()
                .filter(|s| s.content != "[FILTERED DUPLICATE]")
                .collect();

            println!(
                "Actual (non-filtered) submissions: {}",
                actual_submissions.len()
            );
            println!(
                "Filtered duplicate submissions: {}",
                submissions.len() - actual_submissions.len()
            );

            if submissions.len() != 30 {
                println!(
                    "❌ Expected 30 submission elements, but found {}",
                    submissions.len()
                );
                println!("This test failed. Please manually verify the webpage structure.");
                println!("The path used was: html body center table tbody tr td table tbody tr.athing.submission td.title");
                println!("Note: This includes both actual content and filtered duplicates");
                panic!(
                    "Expected 30 submission elements, found {}",
                    submissions.len()
                );
            } else {
                println!("✅ Successfully found exactly 30 submission elements (including filtered duplicates)!");
                println!("✅ SmartCrawler pipeline completed with domain-level filtering applied");
            }
        }
        Err(e) => {
            println!("❌ Failed to crawl Hacker News: {}", e);
            println!("Please verify that:");
            println!("1. A WebDriver server is running on port 4444");
            println!("2. The website https://news.ycombinator.com/ is accessible");
            panic!("Failed to crawl: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Ignored by default, run with: cargo test real_world -- --ignored
#[serial]
async fn test_mykin_ai_team_member() {
    println!("Testing Mykin.ai for Kasper Juul team member using full SmartCrawler pipeline...");

    match full_crawl_pipeline("https://mykin.ai/company").await {
        Ok((tree, storage)) => {
            // Apply domain-level duplicate filtering if available
            let filtered_tree =
                if let Some(domain_duplicates) = storage.get_domain_duplicates("mykin.ai") {
                    println!("Applying domain-level duplicate filtering...");
                    smart_crawler::HtmlParser::filter_domain_duplicates(&tree, domain_duplicates)
                } else {
                    println!("No domain duplicates found, using original tree");
                    tree
                };

            // Find the team member element using the specified path
            let path_to_team_member = "html.w-mod-js.w-mod-ix body div.page-wrapper main.main-wrapper section.section_team div.padding-global div.container-medium div.team_collection.is-desktop.w-dyn-list div.team_collection-list.w-dyn-items div.w-dyn-item a.team_card.w-inline-block div.team_content h4";
            let team_members = filtered_tree.find_by_path(path_to_team_member);

            println!(
                "Found {} team member elements after filtering",
                team_members.len()
            );

            // Print all team members found
            for (i, member) in team_members.iter().enumerate() {
                let content = &member.content;
                let status = if content == "[FILTERED DUPLICATE]" {
                    " (FILTERED)"
                } else {
                    ""
                };
                println!("Team member {}: '{}'{}", i + 1, content, status);
            }

            // Look for Kasper Juul specifically (excluding filtered duplicates)
            let kasper_found = team_members
                .iter()
                .filter(|member| member.content != "[FILTERED DUPLICATE]")
                .any(|member| member.content.contains("Kasper Juul"));

            // Count actual vs filtered team members
            let actual_members: Vec<_> = team_members
                .iter()
                .filter(|m| m.content != "[FILTERED DUPLICATE]")
                .collect();

            println!(
                "Actual (non-filtered) team members: {}",
                actual_members.len()
            );
            println!(
                "Filtered duplicate team members: {}",
                team_members.len() - actual_members.len()
            );

            if !kasper_found {
                println!("❌ Could not find 'Kasper Juul' in team members");
                println!("This test failed. Please manually verify the webpage structure.");
                println!("The path used was: {path_to_team_member}");
                panic!("Could not find Kasper Juul in team members");
            } else {
                println!("✅ Successfully found 'Kasper Juul' in team members!");
                println!("✅ SmartCrawler pipeline completed with domain-level filtering applied");
            }
        }
        Err(e) => {
            println!("❌ Failed to crawl Mykin.ai: {}", e);
            println!("Please verify that:");
            println!("1. A WebDriver server is running on port 4444");
            println!("2. The website https://mykin.ai/company is accessible");
            panic!("Failed to crawl: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Ignored by default
#[serial]
async fn test_webdriver_connection() {
    println!("Testing WebDriver connection...");

    // Initialize crypto provider for rustls
    let _ = rustls::crypto::ring::default_provider().install_default();

    let mut browser = Browser::new(4444);
    match browser.connect().await {
        Ok(()) => {
            println!("✅ Successfully connected to WebDriver");
            let _ = browser.close().await;
        }
        Err(e) => {
            println!("❌ Failed to connect to WebDriver: {}", e);
            println!("Please ensure a WebDriver server is running on port 4444");
            println!("Setup options:");
            println!("   • GeckoDriver: geckodriver (uses port 4444 by default)");
            println!("   • ChromeDriver: chromedriver --port=4444");
            println!("   • Docker: docker run -d -p 4444:4444 selenium/standalone-chrome:latest");
            panic!("WebDriver connection failed: {}", e);
        }
    }
}
