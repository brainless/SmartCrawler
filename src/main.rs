use smart_crawler::{Browser, CliArgs, FetchStatus, HtmlParser, UrlStorage};
use tracing::{debug, error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() {
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

    let mut browser = Browser::new(9515);

    match browser.connect().await {
        Ok(()) => info!("Connected to WebDriver"),
        Err(e) => {
            error!("Failed to connect to WebDriver: {}", e);
            std::process::exit(1);
        }
    }

    let parser = HtmlParser::new();

    for url in &args.links {
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
                            url_data.set_html_data(html_source, html_tree, title);
                            url_data.update_status(FetchStatus::Success);
                        }

                        info!("Successfully processed {}", url);
                    }
                    Err(e) => {
                        error!("Failed to get HTML source for {}: {}", url, e);
                        if let Some(url_data) = storage.get_url_data_mut(url) {
                            url_data.update_status(FetchStatus::Failed(e.to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to navigate to {}: {}", url, e);
                if let Some(url_data) = storage.get_url_data_mut(url) {
                    url_data.update_status(FetchStatus::Failed(e.to_string()));
                }
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
            println!("Title: {}", title);
            println!("Domain: {}", url_data.domain);
            println!("---");
        }
    }

    info!("SmartCrawler finished processing {} URLs", args.links.len());
}
