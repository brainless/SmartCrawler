use crate::content::ScrapedContent;
use fantoccini::{Client, ClientBuilder, Locator};
use futures::future::join_all;
use reqwest::Url;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Browser error: {0}")]
    BrowserError(#[from] fantoccini::error::CmdError),
    #[error("New session error: {0}")]
    NewSessionError(#[from] fantoccini::error::NewSessionError),
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
        let _: () = self
            .client
            .wait()
            .for_url(&Url::parse(url).unwrap())
            .await?;

        let title = self.client.find(Locator::Css("title")).await?;
        let title = title.text().await?;

        let content_selectors = vec![
            "main",
            "article",
            ".content",
            ".main-content",
            "#content",
            "#main",
            ".post-content",
            ".entry-content",
            "body",
        ];

        let mut text_content = String::new();

        for selector_str in content_selectors {
            if let Ok(element) = self.client.find(Locator::Css(selector_str)).await {
                if let Ok(text) = element.text().await {
                    // Remove script and style elements
                    let text: String = text.split_whitespace().collect::<Vec<_>>().join(" ");

                    if text.len() > text_content.len() {
                        text_content = text;
                    }
                }
            }
        }

        // Fallback to body if no content found
        if text_content.trim().is_empty() {
            if let Ok(body_selector) = self.client.find(Locator::Css("body")).await {
                if let Ok(body) = body_selector.text().await {
                    text_content = body.split_whitespace().collect::<Vec<_>>().join(" ");
                }
            }
        }

        let links = self.client.find_all(Locator::Css("a[href]")).await?;
        let links: Vec<String> = join_all(links.iter().map(|link| link.attr("href")))
            .await
            .iter()
            .filter_map(|link| match link {
                Ok(Some(href)) => Some(href.trim().to_string()),
                Ok(None) => None,
                Err(_) => None,
            })
            .collect();

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

        let meta_description = self
            .client
            .find(Locator::Css("meta[name=\"description\"]"))
            .await?;
        let meta_description = meta_description.attr("content").await?;

        let headings = self
            .client
            .find_all(Locator::Css("h1, h2, h3, h4, h5, h6"))
            .await?;
        let headings: Vec<String> = join_all(headings.iter().map(|heading| heading.text()))
            .await
            .iter()
            .filter_map(|heading| match heading {
                Ok(text) => Some(text.trim().to_string()),
                Err(_) => None,
            })
            .filter(|heading| !heading.is_empty())
            .collect();

        Ok(ScrapedContent {
            url: url.to_string(),
            title: Some(title),
            text_content,
            links,
            meta_description: meta_description.map(|s| s.to_string()),
            headings,
        })
    }

    pub async fn scrape_multiple(
        &self,
        urls: &[String],
    ) -> Vec<Result<ScrapedContent, BrowserError>> {
        join_all(urls.iter().map(|url| self.scrape_url(url))).await
    }
}
