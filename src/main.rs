use smart_crawler::{
    Browser, CliArgs, FetchStatus, HtmlParser, TemplateDetector, TemplatePathStore, UrlStorage,
};
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

    info!("Starting SmartCrawler with domain: {}", args.domain);

    let mut storage = UrlStorage::new();
    let mut domain_urls: HashMap<String, HashSet<String>> = HashMap::new();

    // Convert domain to initial URL
    let root_url = smart_crawler::utils::construct_root_url(&args.domain);
    storage.add_url(root_url.clone());
    domain_urls
        .entry(args.domain.clone())
        .or_default()
        .insert(root_url);

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

    // Phase 1: URL Discovery - find additional URLs for each domain
    info!("Starting URL discovery for domains");

    let max_urls_per_domain = if args.prep { 10 } else { 3 };

    // Discover additional URLs for the domain
    let domain = &args.domain;
    let urls = domain_urls.get_mut(domain).unwrap();

    if urls.len() < max_urls_per_domain {
        info!(
            "Domain {} has {} URL(s), searching for more (max: {})...",
            domain,
            urls.len(),
            max_urls_per_domain
        );

        // Pick the first URL to extract links from
        if let Some(first_url) = urls.iter().next() {
            match process_url(&mut browser, &parser, &mut storage, first_url, true).await {
                Ok(html_source) => {
                    let additional_urls = parser.extract_links(&html_source, domain);
                    let mut added_count = 0;

                    for additional_url in additional_urls {
                        if urls.len() >= max_urls_per_domain {
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

    // Phase 2: Process all discovered URLs
    info!("Processing all discovered URLs");

    let mut all_urls: Vec<String> = Vec::new();

    // Collect all URLs with root URL prioritized
    let domain = &args.domain;
    let urls = domain_urls.get(domain).unwrap();
    let root_url = smart_crawler::utils::construct_root_url(domain);

    // Add root URL first
    if urls.contains(&root_url) {
        all_urls.push(root_url.clone());
    }
    // Then add other URLs
    for url in urls {
        if url != &root_url {
            all_urls.push(url.clone());
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

    // Phase 3: Template analysis (prep mode) or standard duplicate analysis
    if args.prep {
        info!("Running template detection analysis in prep mode");
        let mut combined_store = TemplatePathStore::new();
        let template_detector = TemplateDetector::new();

        // Process each completed URL to extract template paths
        let completed_urls = storage.get_completed_urls();
        for url_data in &completed_urls {
            if let Some(html_tree) = &url_data.html_tree {
                let url_store = template_detector.extract_templates_with_paths(html_tree);
                for path in url_store.get_paths() {
                    combined_store.add_path(path.clone());
                }
            }
        }

        let validated_paths = combined_store.get_validated_paths();
        info!(
            "Template analysis complete, found {} total template paths, {} validated (>3 elements)",
            combined_store.get_paths().len(),
            validated_paths.len()
        );
    } else {
        info!("Running standard duplicate analysis");

        let domain = &args.domain;
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

    if args.prep {
        // In prep mode, output detected template paths in serialized format
        println!("\n=== Template Path Detection Results ===");

        let mut combined_store = TemplatePathStore::new();
        let template_detector = TemplateDetector::new();

        // Process each completed URL to extract template paths
        let completed_urls = storage.get_completed_urls();
        if completed_urls.is_empty() {
            println!("No URLs were successfully processed.");
        } else {
            println!(
                "Processed {} URLs for domain {}:",
                completed_urls.len(),
                args.domain
            );
            for url_data in &completed_urls {
                println!(
                    "  - {} ({})",
                    url_data.url,
                    url_data.title.as_deref().unwrap_or("No title")
                );

                if let Some(html_tree) = &url_data.html_tree {
                    let url_store = template_detector.extract_templates_with_paths(html_tree);
                    for path in url_store.get_paths() {
                        combined_store.add_path(path.clone());
                    }
                }
            }

            println!("\nValidated Template Paths (>3 elements, Rust-serializable format):");
            println!("{}", combined_store.to_validated_serialized_string());
        }
    } else {
        // Regular mode - show crawling results
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
                println!("---");
            }
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
