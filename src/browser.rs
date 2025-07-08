use fantoccini::{Client, ClientBuilder};
use serde_json::json;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("WebDriver connection error: {0}")]
    ConnectionError(#[from] fantoccini::error::CmdError),
    #[error("WebDriver not available on port {port}")]
    WebDriverNotAvailable { port: u16 },
    #[error("Failed to extract HTML: {0}")]
    HtmlExtractionError(String),
}

pub struct Browser {
    client: Option<Client>,
    port: u16,
}

impl Browser {
    pub fn new(port: u16) -> Self {
        Browser { client: None, port }
    }

    pub async fn connect(&mut self) -> Result<(), BrowserError> {
        let mut caps = serde_json::map::Map::new();
        let chrome_opts = json!({
            "args": ["--headless", "--no-sandbox", "--disable-dev-shm-usage"]
        });
        caps.insert("goog:chromeOptions".to_string(), chrome_opts);

        let client = ClientBuilder::rustls()
            .map_err(|e| {
                BrowserError::HtmlExtractionError(format!("Failed to create client: {e}"))
            })?
            .capabilities(caps)
            .connect(&format!("http://localhost:{}", self.port))
            .await
            .map_err(|e| {
                if e.to_string().contains("Connection refused") {
                    BrowserError::WebDriverNotAvailable { port: self.port }
                } else {
                    BrowserError::HtmlExtractionError(e.to_string())
                }
            })?;

        self.client = Some(client);
        Ok(())
    }

    pub async fn navigate_to(&mut self, url: &str) -> Result<(), BrowserError> {
        if let Some(client) = &mut self.client {
            client.goto(url).await?;
            tokio::time::sleep(Duration::from_millis(2000)).await;
            Ok(())
        } else {
            Err(BrowserError::HtmlExtractionError(
                "Not connected to browser".to_string(),
            ))
        }
    }

    pub async fn get_html_source(&mut self) -> Result<String, BrowserError> {
        if let Some(client) = &mut self.client {
            let html = client.source().await?;
            Ok(html)
        } else {
            Err(BrowserError::HtmlExtractionError(
                "Not connected to browser".to_string(),
            ))
        }
    }

    pub async fn get_page_title(&mut self) -> Result<String, BrowserError> {
        if let Some(client) = &mut self.client {
            let title = client.title().await?;
            Ok(title)
        } else {
            Err(BrowserError::HtmlExtractionError(
                "Not connected to browser".to_string(),
            ))
        }
    }

    pub async fn close(&mut self) -> Result<(), BrowserError> {
        if let Some(client) = self.client.take() {
            client.close().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_browser_connection_error() {
        rustls::crypto::ring::default_provider()
            .install_default()
            .ok();
        let mut browser = Browser::new(9999); // Non-existent port
        let result = browser.connect().await;
        assert!(result.is_err());
        // Just check that it's an error - the specific error type may vary
        match result.unwrap_err() {
            BrowserError::WebDriverNotAvailable { port } => assert_eq!(port, 9999),
            BrowserError::HtmlExtractionError(_) => {} // Also acceptable
            _ => panic!("Unexpected error type"),
        }
    }

    #[tokio::test]
    async fn test_browser_operations_without_connection() {
        let mut browser = Browser::new(4444);

        let result = browser.navigate_to("https://example.com").await;
        assert!(result.is_err());

        let result = browser.get_html_source().await;
        assert!(result.is_err());

        let result = browser.get_page_title().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_browser_connect_to_example() {
        rustls::crypto::ring::default_provider()
            .install_default()
            .ok();
        let mut browser = Browser::new(4444);

        if browser.connect().await.is_ok() {
            let result = browser.navigate_to("https://example.com").await;
            assert!(result.is_ok());

            let title = browser.get_page_title().await;
            assert!(title.is_ok());

            let html = browser.get_html_source().await;
            assert!(html.is_ok());

            let _ = browser.close().await;
        }
    }
}
