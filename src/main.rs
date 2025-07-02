use dotenv::dotenv;
use smart_crawler::{
    bounding_box::BoundingBoxAnalyzer,
    browser::Browser,
    claude::ClaudeClient, // Import ClaudeClient for instantiation
    cli::{AppMode, BoundingBoxConfig, CleanHtmlConfig, CrawlerConfig},
    content::clean_html_source,
    crawler::SmartCrawler,
};
use std::sync::Arc; // Import Arc
use tracing::{error, info};

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Parse command line arguments
    let app_mode = CrawlerConfig::from_args();

    match app_mode {
        AppMode::CleanHtml(clean_config) => {
            handle_clean_html_mode(clean_config).await;
        }
        AppMode::BoundingBox(bbox_config) => {
            handle_bounding_box_mode(bbox_config).await;
        }
        AppMode::Crawl(config) => {
            handle_crawl_mode(config).await;
        }
    }
}

async fn handle_bounding_box_mode(config: BoundingBoxConfig) {
    // Initialize logging
    if config.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    // Initialize rustls crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    info!("Starting bounding box analysis");
    info!("URL: {}", config.url);
    info!("Tolerance: {} pixels", config.tolerance);

    // Create browser instance
    let browser = match Browser::new().await {
        Ok(browser) => browser,
        Err(e) => {
            error!("Failed to create browser: {}", e);
            eprintln!("❌ Error: Failed to create browser: {}", e);
            eprintln!("Make sure you have a WebDriver server running on http://localhost:4444");
            eprintln!("You can start one using: docker run -d -p 4444:4444 selenium/standalone-chrome");
            std::process::exit(1);
        }
    };

    // Navigate to the URL
    println!("🌐 Navigating to: {}", config.url);
    
    // Create analyzer and perform analysis
    let analyzer = BoundingBoxAnalyzer::new(&browser);
    
    match analyzer.navigate_and_analyze(&config.url, config.tolerance).await {
        Ok(groups) => {
            println!("✅ Bounding box analysis completed!");
            analyzer.print_analysis(&groups);
            
            println!("\n🎨 Visualization has been added to the webpage.");
            println!("📋 Legend is displayed in the top-right corner.");
            println!("🔍 Groups with multiple elements are highlighted with colored borders.");
            println!("⌨️  Press Ctrl+C to exit and close the browser.");
            
            // Keep the browser open for visualization
            tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
            println!("\n👋 Closing browser...");
        }
        Err(e) => {
            error!("Bounding box analysis failed: {}", e);
            eprintln!("❌ Error: {}", e);
            std::process::exit(1);
        }
    }

    // Close browser
    if let Err(e) = browser.close().await {
        error!("Failed to close browser: {}", e);
    }
}

async fn handle_clean_html_mode(config: CleanHtmlConfig) {
    // Initialize logging
    if config.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    // Initialize rustls crypto provider (needed for browser operations)
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Validate configuration
    if let Err(e) = config.validate() {
        error!("Configuration error: {}", e);
        std::process::exit(1);
    }

    info!("Starting HTML cleaning");
    if config.is_url_source() {
        info!("Input URL: {}", config.input_source);
    } else {
        info!("Input file: {}", config.input_source);
    }
    info!("Output file: {}", config.output_file);

    match clean_html_source(&config.input_source, &config.output_file).await {
        Ok(()) => {
            info!("HTML cleaning completed successfully!");
            println!("✅ HTML cleaned successfully!");
            if config.is_url_source() {
                println!("🌐 Input URL: {}", config.input_source);
            } else {
                println!("📄 Input file: {}", config.input_source);
            }
            println!("📄 Output: {}", config.output_file);
        }
        Err(e) => {
            error!("HTML cleaning failed: {}", e);
            eprintln!("❌ Error: {e}");
            std::process::exit(1);
        }
    }
}

async fn handle_crawl_mode(config: CrawlerConfig) {
    // Initialize logging
    if config.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    // Validate configuration
    if let Err(e) = config.validate() {
        error!("Configuration error: {}", e);
        std::process::exit(1);
    }

    info!("Starting Smart Crawler");
    info!("Objective: {}", config.objective);
    info!("Domains: {:?}", config.domains);
    info!("Max URLs per domain: {}", config.max_urls_per_domain);
    info!("Delay between requests: {}ms", config.delay_ms);

    // Instantiate ClaudeClient
    let claude_client = match ClaudeClient::new() {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to create ClaudeClient: {}", e);
            std::process::exit(1);
        }
    };

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Create and run crawler
    // Pass Arc<ClaudeClient> to SmartCrawler::new
    let crawler = match SmartCrawler::new(config, Arc::new(claude_client)).await {
        Ok(crawler) => crawler,
        Err(e) => {
            // CrawlerError now includes LlmError or ClaudeInitializationError
            error!("Failed to initialize crawler: {}", e);
            std::process::exit(1);
        }
    };

    match crawler.crawl_all_domains().await {
        Ok(results) => {
            info!("Crawling completed successfully!");

            // Print results to console
            println!("\n{:=^80}", " CRAWLING RESULTS ");
            println!("Objective: {}", results.objective);
            println!("Domains: {}", results.domains.join(", "));
            println!("\n{:-^80}", " DOMAIN RESULTS ");

            for result in &results.results {
                println!("\nDomain: {}", result.domain);
                println!("URLs Selected: {}", result.selected_urls.len());
                println!("Pages Scraped: {}", result.scraped_content.len());
                println!(
                    "Entities Extracted: {}",
                    result
                        .extracted_entities
                        .iter()
                        .map(|e| e.entity_count())
                        .sum::<usize>()
                );

                println!("Analysis:");
                for analysis_item in &result.analysis {
                    println!("- {analysis_item}");
                }

                if !result.extracted_entities.is_empty() {
                    println!("\nExtracted Entities:");
                    for entity_result in &result.extracted_entities {
                        println!(
                            "  From {}: {} entities (confidence: {:.1}%)",
                            entity_result.url,
                            entity_result.entity_count(),
                            entity_result.extraction_confidence * 100.0
                        );
                    }
                }

                println!("{:-^50}", "");
            }

            // Save results if output file specified
            if let Err(e) = crawler.save_results(&results).await {
                error!("Failed to save results: {}", e);
            }
        }
        Err(e) => {
            error!("Crawling failed: {}", e);
            std::process::exit(1);
        }
    }
}
