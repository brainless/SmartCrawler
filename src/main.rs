use smart_crawler::{Browser, CliArgs, FetchStatus, HtmlParser, UrlStorage};
use std::collections::{HashMap, HashSet};
use tracing::{debug, error, info};

#[tokio::main]
async fn main() {
    // Initialize crypto provider for rustls
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install default crypto provider");

    tracing_subscriber::fmt::init();

    let args = match CliArgs::parse() {
        Ok(args) => args,
        Err(e) => {
            error!("Error parsing arguments: {}", e);
            std::process::exit(1);
        }
    };

    info!("Starting SmartCrawler with {} URLs", args.links.len());

    let mut storage = UrlStorage::new();
    for link in &args.links {
        storage.add_url(link.clone());
    }

    let mut browser = Browser::new(4444);

    match browser.connect().await {
        Ok(()) => info!("Connected to WebDriver"),
        Err(e) => {
            error!("Failed to connect to WebDriver: {}", e);
            eprintln!("\n❌ WebDriver Connection Failed");
            eprintln!("📋 Please ensure a WebDriver server is running on port 4444");
            eprintln!("💡 Quick setup options:");
            eprintln!("   • GeckoDriver: geckodriver (uses port 4444 by default)");
            eprintln!("   • ChromeDriver: chromedriver --port=4444");
            eprintln!("   • Docker: docker run -d -p 4444:4444 selenium/standalone-chrome:latest");
            eprintln!("   • Check status: curl http://localhost:4444/status");
            eprintln!("📖 See CLAUDE.md for detailed setup instructions");
            std::process::exit(1);
        }
    }

    let parser = HtmlParser::new();

    // Phase 1: Preparation stage - fetch additional URLs from same domains
    info!("Starting preparation stage to collect URLs from same domains");

    let mut domain_urls: HashMap<String, HashSet<String>> = HashMap::new();

    // Group initial URLs by domain
    for url in &args.links {
        if let Some(domain) = smart_crawler::utils::extract_domain_from_url(url) {
            domain_urls.entry(domain).or_default().insert(url.clone());
        }
    }

    // Add root URLs for each domain if not already present
    for (domain, urls) in &mut domain_urls {
        let root_url = smart_crawler::utils::construct_root_url(domain);
        if !urls.contains(&root_url) {
            urls.insert(root_url.clone());
            storage.add_url(root_url);
            info!(
                "Added root URL for domain {}: {}",
                domain,
                smart_crawler::utils::construct_root_url(domain)
            );
        }
    }

    // For each domain, try to find additional URLs
    for (domain, urls) in &mut domain_urls {
        if urls.len() < 3 {
            info!(
                "Domain {} has only {} URL(s), searching for more...",
                domain,
                urls.len()
            );

            // Pick the first URL to extract links from
            if let Some(first_url) = urls.iter().next() {
                match process_url(&mut browser, &parser, &mut storage, first_url, true).await {
                    Ok(html_source) => {
                        let additional_urls = parser.extract_links(&html_source, domain);
                        let mut added_count = 0;

                        for additional_url in additional_urls {
                            if urls.len() >= 3 {
                                break;
                            }
                            if urls.insert(additional_url.clone()) {
                                storage.add_url(additional_url);
                                added_count += 1;
                            }
                        }

                        info!(
                            "Found {} additional URLs for domain {}",
                            added_count, domain
                        );
                    }
                    Err(e) => {
                        error!("Failed to extract links from {}: {}", first_url, e);
                    }
                }
            }
        }
    }

    // Phase 2: Process all URLs (initial + discovered) with root URL prioritization
    info!("Processing all URLs with root URL prioritization");

    let mut all_urls: Vec<String> = Vec::new();

    // First, add all user-specified URLs
    for url in &args.links {
        all_urls.push(url.clone());
    }

    // Then, add root URLs for each domain (if not already in user-specified URLs)
    for domain in domain_urls.keys() {
        let root_url = smart_crawler::utils::construct_root_url(domain);
        if !args.links.contains(&root_url) {
            all_urls.push(root_url);
        }
    }

    // Finally, add all other discovered URLs
    for urls in domain_urls.values() {
        for url in urls {
            if !args.links.contains(url) && !smart_crawler::utils::is_root_url(url) {
                all_urls.push(url.clone());
            }
        }
    }

    for url in &all_urls {
        if let Some(url_data) = storage.get_url_data(url) {
            if matches!(url_data.status, FetchStatus::Success) {
                continue; // Already processed
            }
        }

        match process_url(&mut browser, &parser, &mut storage, url, false).await {
            Ok(_) => info!("Successfully processed {}", url),
            Err(e) => error!("Failed to process {}: {}", url, e),
        }
    }

    // Phase 3: Analyze domain duplicates
    info!("Analyzing domain-level duplicate nodes");

    for domain in domain_urls.keys() {
        storage.analyze_domain_duplicates(domain);
        if let Some(duplicates) = storage.get_domain_duplicates(domain) {
            let duplicate_count = duplicates.get_duplicate_count();
            if duplicate_count > 0 {
                info!(
                    "Found {} duplicate node patterns for domain {}",
                    duplicate_count, domain
                );
            } else {
                info!(
                    "No duplicate patterns found for domain {} (likely insufficient pages)",
                    domain
                );
            }
        }
    }

    let _ = browser.close().await;

    println!("\n=== Crawling Results ===");
    let completed_urls = storage.get_completed_urls();

    if completed_urls.is_empty() {
        println!("No URLs were successfully processed.");
    } else {
        for url_data in completed_urls {
            let title = url_data.title.as_deref().unwrap_or("No title found");
            println!("URL: {}", url_data.url);
            println!("Title: {title}");
            println!("Domain: {}", url_data.domain);

            if args.verbose {
                if let Some(html_tree) = &url_data.html_tree {
                    if let Some(domain_duplicates) = storage.get_domain_duplicates(&url_data.domain)
                    {
                        let filtered_tree =
                            HtmlParser::filter_domain_duplicates(html_tree, domain_duplicates);
                        println!("Filtered HTML Tree (showing complete structure with duplicate marking):");
                        print_html_tree(&filtered_tree, 0);
                    } else {
                        println!("HTML Tree (no duplicates to filter):");
                        print_html_tree(html_tree, 0);
                    }
                }
            }

            println!("---");
        }
    }

    info!("SmartCrawler finished processing {} URLs", all_urls.len());
}

async fn process_url(
    browser: &mut Browser,
    parser: &HtmlParser,
    storage: &mut UrlStorage,
    url: &str,
    return_html: bool,
) -> Result<String, String> {
    info!("Processing URL: {}", url);

    if let Some(url_data) = storage.get_url_data_mut(url) {
        url_data.update_status(FetchStatus::InProgress);
    }

    match browser.navigate_to(url).await {
        Ok(()) => {
            debug!("Successfully navigated to {}", url);

            match browser.get_html_source().await {
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
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to navigate: {e}");
            if let Some(url_data) = storage.get_url_data_mut(url) {
                url_data.update_status(FetchStatus::Failed(error_msg.clone()));
            }
            Err(error_msg)
        }
    }
}

fn print_html_tree(node: &smart_crawler::HtmlNode, indent: usize) {
    let indent_str = "  ".repeat(indent);

    // Build the element info string with tag, id, and classes
    let mut element_info = node.tag.clone();
    if let Some(id) = &node.id {
        element_info.push_str(&format!("#{id}"));
    }
    if !node.classes.is_empty() {
        element_info.push_str(&format!("[{}]", node.classes.join(" ")));
    }

    if !node.content.is_empty() {
        println!(
            "{}{}: {}",
            indent_str,
            element_info,
            node.content.chars().take(100).collect::<String>()
        );
    } else {
        println!("{indent_str}{element_info}");
    }

    for child in &node.children {
        print_html_tree(child, indent + 1);
    }
}
