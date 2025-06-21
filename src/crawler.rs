use crate::browser::Browser;
use crate::claude::ClaudeClient;
use crate::cli::CrawlerConfig;
use crate::content::ScrapedContent;
use crate::sitemap::SitemapParser;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrawlerError {
    #[error("Sitemap error: {0}")]
    SitemapError(#[from] crate::sitemap::SitemapError),
    #[error("Claude API error: {0}")]
    ClaudeError(#[from] crate::claude::ClaudeError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Browser error: {0}")]
    BrowserError(#[from] crate::browser::BrowserError),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrawlResult {
    pub domain: String,
    pub objective: String,
    pub selected_urls: Vec<String>,
    pub scraped_content: Vec<ScrapedContent>,
    pub analysis: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrawlerResults {
    pub objective: String,
    pub domains: Vec<String>,
    pub results: Vec<CrawlResult>,
}

pub struct SmartCrawler {
    sitemap_parser: SitemapParser,
    claude_client: ClaudeClient,
    browser: Browser,
    config: CrawlerConfig,
    scraped_urls: Arc<Mutex<HashMap<String, HashSet<String>>>>,
}

impl SmartCrawler {
    pub async fn new(config: CrawlerConfig) -> Result<Self, CrawlerError> {
        let sitemap_parser = SitemapParser::new();
        let claude_client = ClaudeClient::new()?;
        let browser = Browser::new().await?;

        Ok(Self {
            sitemap_parser,
            claude_client,
            browser,
            config,
            scraped_urls: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn crawl_all_domains(&self) -> Result<CrawlerResults, CrawlerError> {
        let mut results = Vec::new();
        let mut processed_domains = std::collections::HashSet::new();

        loop {
            // Get next unprocessed domain
            let next_domain = {
                let domains = self.config.domains.lock().unwrap();
                domains
                    .iter()
                    .find(|domain| !processed_domains.contains(*domain))
                    .cloned()
            };

            let domain = match next_domain {
                Some(d) => d,
                None => break, // No more domains to process
            };

            processed_domains.insert(domain.clone());
            tracing::info!("Starting crawl for domain: {}", domain);

            match self.crawl_domain(&domain).await {
                Ok(result) => {
                    tracing::info!("Successfully crawled domain: {}", domain);
                    results.push(result);
                }
                Err(e) => {
                    tracing::error!("Failed to crawl domain {}: {}", domain, e);
                    // Continue with other domains even if one fails
                    let failed_result = CrawlResult {
                        domain: domain.clone(),
                        objective: self.config.objective.clone(),
                        selected_urls: Vec::new(),
                        scraped_content: Vec::new(),
                        analysis: vec![format!("Failed to crawl: {}", e)],
                    };
                    results.push(failed_result);
                }
            }
        }

        let final_domains = self.config.domains.lock().unwrap().clone();
        Ok(CrawlerResults {
            objective: self.config.objective.clone(),
            domains: final_domains,
            results,
        })
    }

    async fn crawl_domain(&self, domain: &str) -> Result<CrawlResult, CrawlerError> {
        tracing::info!("Discovering sitemap for: {}", domain);

        // Step 1: Get sitemap URLs
        let sitemap_urls = self.sitemap_parser.get_all_urls(domain).await?;
        tracing::info!(
            "Found {} URLs in sitemap for {}",
            sitemap_urls.len(),
            domain
        );

        let urls_to_analyze = if sitemap_urls.is_empty() {
            tracing::info!(
                "No URLs found in sitemap for {}, scraping root URL for links",
                domain
            );
            let root_url = format!("https://{}", domain);

            // Scrape the root URL to extract all links
            match self.browser.scrape_url(&root_url).await {
                Ok(scraped_content) => {
                    let mut discovered_urls = scraped_content.links;

                    // Filter to only include URLs from the same domain
                    discovered_urls.retain(|url| {
                        if let Ok(parsed_url) = url::Url::parse(url) {
                            if let Some(url_domain) = parsed_url.host_str() {
                                return url_domain == domain
                                    || url_domain.ends_with(&format!(".{}", domain));
                            }
                        }
                        false
                    });

                    // Remove duplicates and include the root URL
                    let mut unique_urls: std::collections::HashSet<String> =
                        discovered_urls.into_iter().collect();
                    unique_urls.insert(root_url.clone());

                    let final_urls: Vec<String> = unique_urls.into_iter().collect();
                    tracing::info!(
                        "Discovered {} URLs from root page of {}",
                        final_urls.len(),
                        domain
                    );
                    final_urls
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to scrape root URL {}: {}, using just root URL",
                        root_url,
                        e
                    );
                    vec![root_url]
                }
            }
        } else {
            sitemap_urls
                .iter()
                .map(|u| u.as_ref().to_string())
                .collect()
        };

        // Step 2: Use Claude to select relevant URLs
        tracing::info!("Asking Claude to select relevant URLs for: {}", domain);
        let mut selected_urls = self
            .claude_client
            .select_urls(
                &self.config.objective,
                &urls_to_analyze,
                domain,
                self.config.max_urls_per_domain,
            )
            .await?;

        // Step 2.5: Filter out already scraped URLs and track unique URLs
        {
            let mut scraped_urls = self.scraped_urls.lock().unwrap();
            let domain_urls = scraped_urls
                .entry(domain.to_string())
                .or_insert_with(HashSet::new);

            selected_urls.retain(|url| {
                if domain_urls.contains(url) {
                    tracing::debug!("Skipping already scraped URL: {}", url);
                    false
                } else {
                    domain_urls.insert(url.clone());
                    true
                }
            });
        }

        tracing::info!(
            "Selected {} unique URLs for {} (after deduplication)",
            selected_urls.len(),
            domain
        );

        if selected_urls.is_empty() {
            return Ok(CrawlResult {
                domain: domain.to_string(),
                objective: self.config.objective.clone(),
                selected_urls: Vec::new(),
                scraped_content: Vec::new(),
                analysis: vec!["No relevant URLs selected by Claude".to_string()],
            });
        }

        // Step 3: Scrape the selected URLs
        tracing::info!(
            "Scraping {} selected URLs for {}",
            selected_urls.len(),
            domain
        );
        let scrape_results = self.browser.scrape_multiple(&selected_urls).await;

        let mut scraped_content = Vec::new();
        let mut analysis = Vec::new();

        // Step 4: Analyze each scraped page with Claude
        for (i, result) in scrape_results.into_iter().enumerate() {
            match result {
                Ok(content) => {
                    tracing::info!("Analyzing content from: {}", content.url);

                    // Ask Claude to analyze the content
                    match self
                        .claude_client
                        .analyze_content(
                            &self.config.objective,
                            &content.url,
                            &content.text_content,
                        )
                        .await
                    {
                        Ok(analysis_result) => {
                            analysis.push(format!(
                                "URL: {}\nAnalysis: {}",
                                content.url, analysis_result
                            ));
                        }
                        Err(e) => {
                            tracing::warn!("Failed to analyze content from {}: {}", content.url, e);
                            analysis.push(format!("URL: {}\nAnalysis failed: {}", content.url, e));
                        }
                    }

                    scraped_content.push(content);
                }
                Err(e) => {
                    tracing::warn!("Failed to scrape URL {}: {}", selected_urls[i], e);
                    analysis.push(format!("URL: {}\nScraping failed: {}", selected_urls[i], e));
                }
            }
        }

        Ok(CrawlResult {
            domain: domain.to_string(),
            objective: self.config.objective.clone(),
            selected_urls,
            scraped_content,
            analysis,
        })
    }

    pub async fn save_results(&self, results: &CrawlerResults) -> Result<(), CrawlerError> {
        if let Some(output_file) = &self.config.output_file {
            let json_output = serde_json::to_string_pretty(results)?;
            std::fs::write(output_file, json_output)?;
            tracing::info!("Results saved to: {}", output_file);
        }
        Ok(())
    }
}
