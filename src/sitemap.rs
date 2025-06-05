use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SitemapError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("XML parsing failed: {0}")]
    XmlError(#[from] quick_xml::Error),
    #[error("URL parsing failed: {0}")]
    UrlError(#[from] url::ParseError),
    #[error("No sitemap found for domain")]
    NoSitemapFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapUrl {
    pub loc: String,
    pub lastmod: Option<String>,
    pub changefreq: Option<String>,
    pub priority: Option<f32>,
}

impl AsRef<str> for SitemapUrl {
    fn as_ref(&self) -> &str {
        &self.loc
    }
}

#[derive(Debug)]
pub struct SitemapParser {
    client: Client,
}

impl SitemapParser {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Smart-Crawler/1.0")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub async fn discover_sitemap(&self, domain: &str) -> Result<Vec<String>, SitemapError> {
        let mut sitemaps = Vec::new();
        
        // Try common sitemap locations
        let sitemap_urls = vec![
            format!("https://{}/sitemap.xml", domain),
            format!("https://{}/sitemap_index.xml", domain),
            format!("https://{}/sitemaps.xml", domain),
            format!("http://{}/sitemap.xml", domain),
        ];

        for url in sitemap_urls {
            if let Ok(response) = self.client.get(&url).send().await {
                if response.status().is_success() {
                    sitemaps.push(url);
                    break;
                }
            }
        }

        // Try robots.txt
        if sitemaps.is_empty() {
            if let Ok(robots_sitemaps) = self.check_robots_txt(domain).await {
                sitemaps.extend(robots_sitemaps);
            }
        }

        if sitemaps.is_empty() {
            return Err(SitemapError::NoSitemapFound);
        }

        Ok(sitemaps)
    }

    async fn check_robots_txt(&self, domain: &str) -> Result<Vec<String>, SitemapError> {
        let robots_url = format!("https://{}/robots.txt", domain);
        let response = self.client.get(&robots_url).send().await?;
        
        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let content = response.text().await?;
        let mut sitemaps = Vec::new();

        for line in content.lines() {
            if line.to_lowercase().starts_with("sitemap:") {
                if let Some(sitemap_url) = line.split(':').nth(1) {
                    let sitemap_url = sitemap_url.trim();
                    if sitemap_url.starts_with("http") {
                        sitemaps.push(sitemap_url.to_string());
                    }
                }
            }
        }

        Ok(sitemaps)
    }

    pub async fn parse_sitemap(&self, sitemap_url: &str) -> Result<Vec<SitemapUrl>, SitemapError> {
        let response = self.client.get(sitemap_url).send().await?;
        let content = response.text().await?;
        
        let mut urls = Vec::new();
        let mut reader = quick_xml::Reader::from_str(&content);
        reader.trim_text(true);
        
        let mut buf = Vec::new();
        let mut current_url = SitemapUrl {
            loc: String::new(),
            lastmod: None,
            changefreq: None,
            priority: None,
        };
        let mut in_url = false;
        let mut current_tag = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"url" => {
                            in_url = true;
                            current_url = SitemapUrl {
                                loc: String::new(),
                                lastmod: None,
                                changefreq: None,
                                priority: None,
                            };
                        }
                        b"loc" | b"lastmod" | b"changefreq" | b"priority" => {
                            current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                        }
                        _ => {}
                    }
                }
                Ok(quick_xml::events::Event::Text(e)) => {
                    if in_url {
                        let text = e.unescape()?.to_string();
                        match current_tag.as_str() {
                            "loc" => current_url.loc = text,
                            "lastmod" => current_url.lastmod = Some(text),
                            "changefreq" => current_url.changefreq = Some(text),
                            "priority" => current_url.priority = text.parse().ok(),
                            _ => {}
                        }
                    }
                }
                Ok(quick_xml::events::Event::End(ref e)) => {
                    match e.name().as_ref() {
                        b"url" => {
                            if in_url && !current_url.loc.is_empty() {
                                urls.push(current_url.clone());
                            }
                            in_url = false;
                        }
                        b"loc" | b"lastmod" | b"changefreq" | b"priority" => {
                            current_tag.clear();
                        }
                        _ => {}
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => return Err(SitemapError::XmlError(e)),
                _ => {}
            }
            buf.clear();
        }

        // Check if this is a sitemap index
        if urls.is_empty() {
            if let Ok(sitemap_urls) = self.parse_sitemap_index(&content).await {
                for sitemap_url in sitemap_urls {
                    if let Ok(mut sub_urls) = Box::pin(self.parse_sitemap(&sitemap_url)).await {
                        urls.append(&mut sub_urls);
                    }
                }
            }
        }

        Ok(urls)
    }

    async fn parse_sitemap_index(&self, content: &str) -> Result<Vec<String>, SitemapError> {
        let mut sitemap_urls = Vec::new();
        let mut reader = quick_xml::Reader::from_str(content);
        reader.trim_text(true);
        
        let mut buf = Vec::new();
        let mut in_sitemap = false;
        let mut current_tag = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"sitemap" => in_sitemap = true,
                        b"loc" => current_tag = "loc".to_string(),
                        _ => {}
                    }
                }
                Ok(quick_xml::events::Event::Text(e)) => {
                    if in_sitemap && current_tag == "loc" {
                        let url = e.unescape()?.to_string();
                        sitemap_urls.push(url);
                    }
                }
                Ok(quick_xml::events::Event::End(ref e)) => {
                    match e.name().as_ref() {
                        b"sitemap" => in_sitemap = false,
                        b"loc" => current_tag.clear(),
                        _ => {}
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => return Err(SitemapError::XmlError(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(sitemap_urls)
    }

    pub async fn get_all_urls(&self, domain: &str) -> Result<Vec<SitemapUrl>, SitemapError> {
        let sitemaps = self.discover_sitemap(domain).await?;
        let mut all_urls = Vec::new();
        let mut seen_urls = HashSet::new();

        for sitemap_url in sitemaps {
            if let Ok(urls) = self.parse_sitemap(&sitemap_url).await {
                for url in urls {
                    if seen_urls.insert(url.loc.clone()) {
                        all_urls.push(url);
                    }
                }
            }
        }

        Ok(all_urls)
    }
}