use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("CSS selector parsing failed: {0}")]
    SelectorError(String),
    #[error("Content extraction failed")]
    ExtractionError,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScrapedContent {
    pub url: String,
    pub title: Option<String>,
    pub text_content: String,
    pub links: Vec<String>,
    pub meta_description: Option<String>,
    pub headings: Vec<String>,
}

#[derive(Debug)]
pub struct WebScraper {
    client: Client,
    delay_ms: u64,
}

impl WebScraper {
    pub fn new(delay_ms: u64) -> Self {
        let client = Client::builder()
            .user_agent("Smart-Crawler/1.0 (Conservative Crawler)")
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, delay_ms }
    }

    pub async fn scrape_url(&self, url: &str) -> Result<ScrapedContent, ScraperError> {
        // Rate limiting
        if self.delay_ms > 0 {
            sleep(Duration::from_millis(self.delay_ms)).await;
        }

        tracing::info!("Scraping URL: {}", url);

        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(ScraperError::HttpError(reqwest::Error::from(
                response.error_for_status().unwrap_err()
            )));
        }

        let html_content = response.text().await?;
        let document = Html::parse_document(&html_content);

        let content = self.extract_content(&document, url)?;
        
        tracing::info!("Successfully scraped: {} ({})", url, content.text_content.len());
        
        Ok(content)
    }

    fn extract_content(&self, document: &Html, url: &str) -> Result<ScrapedContent, ScraperError> {
        // Extract title
        let title_selector = Selector::parse("title")
            .map_err(|e| ScraperError::SelectorError(e.to_string()))?;
        let title = document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string());

        // Extract meta description
        let meta_desc_selector = Selector::parse("meta[name=\"description\"]")
            .map_err(|e| ScraperError::SelectorError(e.to_string()))?;
        let meta_description = document
            .select(&meta_desc_selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.to_string());

        // Extract headings
        let heading_selector = Selector::parse("h1, h2, h3, h4, h5, h6")
            .map_err(|e| ScraperError::SelectorError(e.to_string()))?;
        let headings: Vec<String> = document
            .select(&heading_selector)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Extract main text content
        let content_selectors = vec![
            "main",
            "article", 
            ".content",
            ".main-content",
            "#content",
            "#main",
            ".post-content",
            ".entry-content",
            "body"
        ];

        let mut text_content = String::new();
        
        for selector_str in content_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    // Remove script and style elements
                    let _script_selector = Selector::parse("script, style, nav, footer, .nav, .footer")
                        .map_err(|e| ScraperError::SelectorError(e.to_string()))?;
                    
                    let text: String = element
                        .text()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ");
                    
                    if text.len() > text_content.len() {
                        text_content = text;
                    }
                }
            }
        }

        // Fallback to body if no content found
        if text_content.trim().is_empty() {
            if let Ok(body_selector) = Selector::parse("body") {
                if let Some(body) = document.select(&body_selector).next() {
                    text_content = body
                        .text()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ");
                }
            }
        }

        // Extract links
        let link_selector = Selector::parse("a[href]")
            .map_err(|e| ScraperError::SelectorError(e.to_string()))?;
        let links: Vec<String> = document
            .select(&link_selector)
            .filter_map(|el| el.value().attr("href"))
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

        Ok(ScrapedContent {
            url: url.to_string(),
            title,
            text_content: text_content.trim().to_string(),
            links,
            meta_description,
            headings,
        })
    }

    pub async fn scrape_multiple(&self, urls: &[String]) -> Vec<Result<ScrapedContent, ScraperError>> {
        let mut results = Vec::new();
        
        for url in urls {
            let result = self.scrape_url(url).await;
            results.push(result);
            
            // Conservative delay between requests
            if self.delay_ms > 0 {
                sleep(Duration::from_millis(self.delay_ms)).await;
            }
        }
        
        results
    }
}