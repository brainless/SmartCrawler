use crate::content::{extract_structured_data, ScrapedWebPage};
use fantoccini::{Client, ClientBuilder};
use scraper::{Html, Selector};
use serde_json::Value;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::time::sleep;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Browser error: {0}")]
    BrowserError(#[from] fantoccini::error::CmdError),
    #[error("New session error: {0}")]
    NewSessionError(#[from] fantoccini::error::NewSessionError),
    #[error("HTML conversion error: {0}")]
    ConversionError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct Browser {
    client: Client,
}

impl Browser {
    pub async fn new() -> Result<Self, BrowserError> {
        let client = ClientBuilder::rustls()?
            .connect("http://localhost:4444")
            .await?;
        Ok(Self { client })
    }

    pub async fn scrape_url(&self, url: &str) -> Result<ScrapedWebPage, BrowserError> {
        self.client.goto(url).await?;

        // Wait for initial page load
        sleep(Duration::from_millis(1000)).await;

        // Perform human-like scrolling to load dynamic content
        self.scroll_page_gradually().await?;

        let html = self
            .client
            .execute("return document.documentElement.outerHTML;", vec![])
            .await?;
        let html = match html {
            Value::String(html) => html,
            _ => {
                return Err(BrowserError::ConversionError(
                    "HTML conversion error".to_string(),
                ))
            }
        };
        let scraper = Html::parse_document(&html);

        let title = scraper
            .select(&Selector::parse("title").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<Vec<_>>()
            .join(" ");

        let content = extract_structured_data(&html).await;

        let links = scraper
            .select(&Selector::parse("a[href]").unwrap())
            .filter_map(|link| link.value().attr("href"))
            .collect::<Vec<_>>();
        let links: Vec<String> = links
            .iter()
            .map(|href| {
                if href.starts_with("http") {
                    href.to_string()
                } else if href.starts_with("//") {
                    format!("https:{}", href)
                } else if href.starts_with('/') {
                    if let Ok(base_url) = url::Url::parse(url) {
                        if let Some(domain) = base_url.host_str() {
                            return format!("{}://{}{}", base_url.scheme(), domain, href);
                        }
                    }
                    href.to_string()
                } else {
                    href.to_string()
                }
            })
            .collect();

        let meta_description = scraper
            .select(&Selector::parse("meta[name=\"description\"]").unwrap())
            .filter_map(|meta| meta.value().attr("content"))
            .collect::<Vec<_>>()
            .join(" ");

        let headings = scraper
            .select(&Selector::parse("h1, h2, h3, h4, h5, h6").unwrap())
            .map(|heading| {
                heading
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string()
            })
            .filter(|heading| !heading.is_empty())
            .collect::<Vec<_>>();

        Ok(ScrapedWebPage {
            url: url.to_string(),
            title: Some(title),
            content,
            links,
            meta_description: Some(meta_description),
            headings,
        })
    }

    /// Scrolls through the page gradually to mimic human behavior and trigger dynamic content loading
    /// Scrolls for a maximum of 10 seconds with realistic pauses between scroll actions
    async fn scroll_page_gradually(&self) -> Result<(), BrowserError> {
        let start_time = Instant::now();
        let max_scroll_duration = Duration::from_secs(10);
        let scroll_step = 300; // pixels to scroll per step
        let scroll_delay = Duration::from_millis(500); // delay between scroll actions

        tracing::debug!("Starting gradual page scroll");

        loop {
            // Check if we've exceeded the time limit
            if start_time.elapsed() >= max_scroll_duration {
                tracing::debug!("Scroll time limit reached (10 seconds), stopping");
                break;
            }

            // Get current scroll position and page height
            let scroll_info = self.client.execute(
                "return { scrollY: window.scrollY, documentHeight: document.documentElement.scrollHeight, windowHeight: window.innerHeight };",
                vec![]
            ).await?;

            let scroll_info = match scroll_info {
                Value::Object(obj) => obj,
                _ => {
                    tracing::warn!("Failed to get scroll information, stopping scroll");
                    break;
                }
            };

            let current_scroll = scroll_info
                .get("scrollY")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as i32;
            let document_height = scroll_info
                .get("documentHeight")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as i32;
            let window_height = scroll_info
                .get("windowHeight")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as i32;

            // Check if we've reached the bottom of the page
            if current_scroll + window_height >= document_height - 50 {
                // 50px buffer
                tracing::debug!("Reached bottom of page, stopping scroll");
                break;
            }

            // Scroll down by the step amount
            let scroll_script = format!("window.scrollBy(0, {});", scroll_step);
            if let Err(e) = self.client.execute(&scroll_script, vec![]).await {
                tracing::warn!("Failed to execute scroll command: {}", e);
                break;
            }

            tracing::debug!(
                "Scrolled to position: {} (page height: {})",
                current_scroll + scroll_step,
                document_height
            );

            // Wait before next scroll to mimic human behavior
            sleep(scroll_delay).await;
        }

        tracing::debug!(
            "Completed gradual page scroll in {:?}",
            start_time.elapsed()
        );
        Ok(())
    }

    pub async fn close(self) -> Result<(), BrowserError> {
        self.client.close().await?;
        Ok(())
    }
}
