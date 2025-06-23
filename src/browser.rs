use crate::content::{extract_structured_data, ScrapedContent};
use fantoccini::{Client, ClientBuilder};
use futures::future::join_all;
use scraper::{Html, Selector};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Browser error: {0}")]
    BrowserError(#[from] fantoccini::error::CmdError),
    #[error("New session error: {0}")]
    NewSessionError(#[from] fantoccini::error::NewSessionError),
    #[error("HTML conversion error: {0}")]
    ConversionError(String),
    #[error("Content extraction error: {0}")]
    DomContentExtractionError(#[from] dom_content_extraction::DomExtractionError),
}

pub struct Browser {
    client: Client,
}

impl Browser {
    pub async fn new() -> Result<Self, BrowserError> {
        let client = ClientBuilder::native()
            .connect("http://localhost:4444")
            .await?;
        Ok(Self { client })
    }

    pub async fn scrape_url(&self, url: &str) -> Result<ScrapedContent, BrowserError> {
        self.client.goto(url).await?;

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
                heading.text().collect::<Vec<_>>().join(" ").trim().to_string()
            })
            .filter(|heading| !heading.is_empty())
            .collect::<Vec<_>>();

        Ok(ScrapedContent {
            url: url.to_string(),
            title: Some(title),
            content,
            links,
            meta_description: Some(meta_description),
            headings,
        })
    }

    pub async fn scrape_multiple(
        &self,
        urls: &[String],
    ) -> Vec<Result<ScrapedContent, BrowserError>> {
        join_all(urls.iter().map(|url| self.scrape_url(url))).await
    }

    pub async fn close(self) -> Result<(), BrowserError> {
        self.client.close().await?;
        Ok(())
    }
}
