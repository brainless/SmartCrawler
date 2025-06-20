use smart_crawler::{cli::CrawlerConfig, crawler::SmartCrawler};
use tracing::{error, info};
use tracing_subscriber;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Parse command line arguments
    let config = CrawlerConfig::from_args();

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

    // Create and run crawler
    let crawler = match SmartCrawler::new(config).await {
        Ok(crawler) => crawler,
        Err(e) => {
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
                println!("Analysis:");
                for analysis_item in &result.analysis {
                    println!("- {}", analysis_item);
                }

                println!("\n{:-^50}", " SCRAPED CONTENT DETAILS ");
                if result.scraped_content.is_empty() {
                    println!("No content scraped for this domain.");
                } else {
                    for (idx, content) in result.scraped_content.iter().enumerate() {
                        println!("\n[{}] URL: {}", idx + 1, content.url);
                        if let Some(title) = &content.title {
                            println!("    Title: {}", title);
                        }
                        let snippet = content.text_content.chars().take(200).collect::<String>();
                        println!("    Content Snippet: {}...", snippet);
                        if idx < result.scraped_content.len() - 1 {
                            println!("    {:-^40}", ""); // Separator between content items
                        }
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
