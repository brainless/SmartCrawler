use crate::browser::Browser;
use crate::cli::CrawlerConfig;
use crate::content::ScrapedWebPage;
use crate::entities::EntityExtractionResult;
use crate::llm::{LlmError, LLM};
use crate::sitemap::SitemapParser;
use crate::url_ranking::UrlRanker;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum CrawlerError {
    #[error("Sitemap error: {0}")]
    SitemapError(#[from] crate::sitemap::SitemapError),
    // #[error("Claude API error: {0}")]  // Will be replaced by LlmError
    // ClaudeError(#[from] crate::claude::ClaudeError), // This might be removed if ClaudeError is not directly exposed
    #[error("LLM error: {0}")]
    LlmError(String), // Store as string for simplicity, or use Box<dyn Error>
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Browser error: {0}")]
    BrowserError(#[from] crate::browser::BrowserError),
    #[error("Claude client initialization error: {0}")] // Specific error for ClaudeClient::new()
    ClaudeInitializationError(#[from] crate::claude::ClaudeError),
}

// Implement From<LlmError> for CrawlerError
impl From<LlmError> for CrawlerError {
    fn from(err: LlmError) -> Self {
        CrawlerError::LlmError(err.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrawlResult {
    pub domain: String,
    pub objective: String,
    pub selected_urls: Vec<String>,
    pub scraped_content: Vec<ScrapedWebPage>,
    pub analysis: Vec<String>,
    pub extracted_entities: Vec<EntityExtractionResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrawlerResults {
    pub objective: String,
    pub domains: Vec<String>,
    pub results: Vec<CrawlResult>,
}

/// Extract path and query from a URL
fn extract_path_query(url: &str) -> Option<String> {
    if let Ok(parsed) = Url::parse(url) {
        let mut path_query = parsed.path().to_string();
        if let Some(query) = parsed.query() {
            path_query.push('?');
            path_query.push_str(query);
        }
        Some(path_query)
    } else {
        None
    }
}

/// Check if a URL path contains objective keywords
fn matches_objective_keywords(path_query: &str, objective: &str) -> bool {
    // Extract keywords from objective (simple approach - split by common separators)
    let objective_lower = objective.to_lowercase();
    let objective_keywords: Vec<&str> = objective_lower
        .split_whitespace()
        .filter(|word| word.len() > 2) // Filter out short words
        .collect();

    let path_lower = path_query.to_lowercase();

    // Check if any objective keyword appears in the path
    objective_keywords
        .iter()
        .any(|keyword| path_lower.contains(keyword))
}

pub struct SmartCrawler {
    sitemap_parser: SitemapParser,
    llm_client: Arc<dyn LLM + Send + Sync>, // Changed from claude_client: ClaudeClient
    config: CrawlerConfig,
    urls_scraped: Arc<Mutex<HashMap<String, HashSet<String>>>>, // domain -> set of path+query
}

impl SmartCrawler {
    // Updated constructor to accept Arc<dyn LLM + Send + Sync>
    pub async fn new(
        config: CrawlerConfig,
        llm_client: Arc<dyn LLM + Send + Sync>,
    ) -> Result<Self, CrawlerError> {
        let sitemap_parser = SitemapParser::new();

        Ok(Self {
            sitemap_parser,
            llm_client, // Use the passed llm_client
            config,
            urls_scraped: Arc::new(Mutex::new(HashMap::new())),
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
                        extracted_entities: Vec::new(),
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
        let browser = Browser::new().await?;

        let mut urls_to_analyze: Vec<String> = sitemap_urls
            .iter()
            .map(|u| u.as_ref().to_string())
            .collect();

        let urls_found_in_homepage: Vec<String> = {
            tracing::info!(
                "No URLs found in sitemap for {}, scraping root URL for links",
                domain
            );
            let root_url = format!("https://{domain}");

            // Scrape the root URL to extract all links
            match browser.scrape_url(&root_url).await {
                Ok(scraped_content) => {
                    let mut discovered_urls = scraped_content.links;

                    // Filter to only include URLs from the same domain
                    discovered_urls.retain(|url| {
                        if let Ok(parsed_url) = url::Url::parse(url) {
                            if let Some(url_domain) = parsed_url.host_str() {
                                return url_domain == domain
                                    || url_domain.ends_with(&format!(".{domain}"));
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
        };
        urls_to_analyze.extend(urls_found_in_homepage);

        // Only retain URLs that are one level deeper
        let urls_one_level_deeper =
            select_urls_one_level_deeper(urls_to_analyze.clone(), format!("https://{domain}"));

        // Step 1.5: Add URLs that match objective keywords (improvement from issue #19)
        let objective_matching_urls: Vec<String> = urls_to_analyze
            .iter()
            .filter(|url| {
                if let Some(path_query) = extract_path_query(url) {
                    matches_objective_keywords(&path_query, &self.config.objective)
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        let mut urls_to_analyze = Vec::new();
        if !objective_matching_urls.is_empty() {
            tracing::info!(
                "Found {} URLs matching objective keywords for {}",
                objective_matching_urls.len(),
                domain
            );
            // Add objective matching URLs first to ensure they get higher priority
            urls_to_analyze.extend(objective_matching_urls);
        }
        // Add the remaining URLs from urls_one_level_deeper
        urls_to_analyze.extend(urls_one_level_deeper);

        // Step 2: URL Selection (with optional keyword-based pre-filtering)
        let mut selected_urls = if self.config.enable_keyword_filtering {
            // Two-stage selection: keyword ranking + LLM selection
            tracing::info!(
                "Using two-stage URL selection (keywords + LLM) for: {}",
                domain
            );

            // Stage 1: Generate keywords and rank URLs
            let keywords = self
                .llm_client
                .generate_keywords(&self.config.objective, domain)
                .await?;

            let url_ranker = UrlRanker::new(self.config.url_ranking_config.clone());
            let top_candidates =
                url_ranker.rank_urls(&urls_to_analyze, &keywords, self.config.max_urls_per_domain);

            tracing::info!(
                "Generated {} keywords, ranked {} URLs to {} candidates for LLM selection",
                keywords.len(),
                urls_to_analyze.len(),
                top_candidates.len()
            );

            // Stage 2: LLM selection from top candidates
            if top_candidates.is_empty() {
                Vec::new()
            } else {
                self.llm_client
                    .select_urls(
                        &self.config.objective,
                        &top_candidates,
                        domain,
                        self.config.max_urls_per_domain,
                    )
                    .await?
            }
        } else {
            // Traditional single-stage LLM selection
            tracing::info!("Using traditional LLM-only URL selection for: {}", domain);
            self.llm_client
                .select_urls(
                    &self.config.objective,
                    &urls_to_analyze,
                    domain,
                    self.config.max_urls_per_domain,
                )
                .await?
        };

        // Step 2.5: Filter out already scraped URLs and track unique URLs (using path+query only)
        {
            let mut scraped_urls = self.urls_scraped.lock().unwrap();
            let domain_paths = scraped_urls.entry(domain.to_string()).or_default();

            selected_urls.retain(|url: &String| {
                if let Some(path_query) = extract_path_query(url) {
                    if domain_paths.contains(&path_query) {
                        tracing::debug!("Skipping already scraped path: {}", path_query);
                        false
                    } else {
                        domain_paths.insert(path_query);
                        true
                    }
                } else {
                    tracing::warn!("Failed to parse URL for deduplication: {}", url);
                    true // Include URLs we can't parse to be safe
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
                analysis: vec!["No relevant URLs selected by LLM".to_string()], // Changed Claude to LLM
                extracted_entities: Vec::new(),
            });
        }

        // Step 3: Scrape the selected URLs one at a time
        tracing::info!(
            "Scraping {} selected URLs for {}",
            selected_urls.len(),
            domain
        );

        let mut scraped_content = Vec::new();
        let mut analysis = Vec::new();
        let mut extracted_entities = Vec::new();
        let mut objective_met = false;

        // Step 4: Scrape and analyze each URL sequentially
        for url in &selected_urls {
            if objective_met {
                // Ask user if they want to continue
                println!("\nObjective has been met! Current analysis:");
                for entry in &analysis {
                    println!("{entry}");
                }

                print!("\nContinue crawling remaining URLs? (y/N): ");
                std::io::stdout().flush().unwrap();

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let continue_crawling = input.trim().to_lowercase() == "y";

                if !continue_crawling {
                    break;
                }
            }

            match browser.scrape_url(url).await {
                Ok(web_page) => {
                    tracing::info!("Analyzing content from: {}", web_page.url);

                    // Extract structured entities from the content
                    match self
                        .llm_client
                        .extract_entities(
                            &self.config.objective,
                            &web_page.url,
                            &web_page.content.to_prompt(),
                        )
                        .await
                    {
                        Ok(entity_result) => {
                            let entity_count = entity_result.entity_count();
                            let confidence = entity_result.extraction_confidence;

                            analysis.push(format!(
                                "URL: {}\nExtracted {} entities with {:.1}% confidence\nAnalysis: {}",
                                web_page.url,
                                entity_count,
                                confidence * 100.0,
                                entity_result.raw_analysis
                            ));

                            extracted_entities.push(entity_result);
                            objective_met = entity_count > 0 && confidence > 0.5;
                        }
                        Err(e) => {
                            tracing::warn!("Entity extraction failed for {}: {}", web_page.url, e);

                            // Fallback to simple content analysis
                            match self
                                .llm_client
                                .analyze_content(
                                    &self.config.objective,
                                    &web_page.url,
                                    &web_page.content.to_prompt(),
                                )
                                .await
                            {
                                Ok(llm_response) => {
                                    if llm_response.is_objective_met {
                                        let results_count = llm_response
                                            .results
                                            .as_ref()
                                            .map(|r| r.len())
                                            .unwrap_or(0);
                                        analysis.push(format!(
                                            "URL: {}\nObjective Met: {}\nFound {} results\nAnalysis: {}",
                                            web_page.url,
                                            llm_response.is_objective_met,
                                            results_count,
                                            llm_response.analysis.unwrap_or_else(|| "No analysis provided".to_string())
                                        ));
                                        objective_met = true;
                                    } else {
                                        analysis.push(format!(
                                            "URL: {}\nObjective Not Met\nAnalysis: {}",
                                            web_page.url,
                                            llm_response.analysis.unwrap_or_else(|| {
                                                "No analysis provided".to_string()
                                            })
                                        ));
                                        objective_met = false;
                                    }
                                }
                                Err(_) => {
                                    objective_met = false;
                                }
                            }
                        }
                    }

                    scraped_content.push(web_page);
                }
                Err(e) => {
                    tracing::warn!("Failed to scrape URL {}: {}", url, e);
                    analysis.push(format!("URL: {url}\nScraping failed: {e}"));
                }
            }
        }

        browser.close().await?;

        Ok(CrawlResult {
            domain: domain.to_string(),
            objective: self.config.objective.clone(),
            selected_urls,
            scraped_content,
            analysis,
            extracted_entities,
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

/// Retains URLs that are one level deeper than the base_url, ignoring the scheme of the URLs and base_url
///
/// Examples:
/// urls = ["https://example.com/level1/level2", "https://example.com/level1/level2/level3", "https://example.com/different-level1/level2/level3/level4"]
/// base_url = "https://example.com"
/// Output: ["https://example.com/level1", "https://example.com/different-level1"]
///
/// urls = ["https://example.com/different-level1/level2", "https://example.com/level1/level2/level3", "https://example.com/level1/level2/level3/level4"]
/// base_url = "https://example.com/level1"
/// Output: ["https://example.com/level1/level2"]
fn select_urls_one_level_deeper<T: AsRef<str>>(urls: Vec<T>, base_url: T) -> Vec<String> {
    let base_url_str = base_url.as_ref();

    // Parse the base URL to get the path
    let base_parsed = match url::Url::parse(base_url_str) {
        Ok(parsed) => parsed,
        Err(_) => return Vec::new(),
    };

    let base_domain = match base_parsed.host_str() {
        Some(domain) => domain,
        None => return Vec::new(),
    };

    let base_path = base_parsed.path().trim_end_matches('/');
    let base_segments: Vec<&str> = if base_path.is_empty() || base_path == "/" {
        Vec::new()
    } else {
        base_path.trim_start_matches('/').split('/').collect()
    };

    let mut result = Vec::new();
    let mut seen = HashSet::new();

    for url in urls {
        let url_str = url.as_ref();

        if let Ok(parsed_url) = url::Url::parse(url_str) {
            // Check if URL is from the same domain
            if let Some(url_domain) = parsed_url.host_str() {
                if url_domain != base_domain && !url_domain.ends_with(&format!(".{base_domain}")) {
                    continue;
                }

                let url_path = parsed_url.path().trim_end_matches('/');
                let url_segments: Vec<&str> = if url_path.is_empty() || url_path == "/" {
                    Vec::new()
                } else {
                    url_path.trim_start_matches('/').split('/').collect()
                };

                // Check if URL starts with the base path and has at least one more segment
                if url_segments.len() > base_segments.len() {
                    let mut matches_base = true;
                    for (i, base_segment) in base_segments.iter().enumerate() {
                        if i >= url_segments.len() || url_segments[i] != *base_segment {
                            matches_base = false;
                            break;
                        }
                    }

                    if matches_base {
                        // Construct the URL that is one level deeper than base
                        let mut one_level_deeper_segments = base_segments.clone();
                        one_level_deeper_segments.push(url_segments[base_segments.len()]);

                        let one_level_deeper_path = if one_level_deeper_segments.is_empty() {
                            String::new()
                        } else {
                            format!("/{}", one_level_deeper_segments.join("/"))
                        };

                        let one_level_deeper_url = format!(
                            "{}://{}{}",
                            parsed_url.scheme(),
                            parsed_url.host_str().unwrap_or(""),
                            one_level_deeper_path
                        );

                        if seen.insert(one_level_deeper_url.clone()) {
                            result.push(one_level_deeper_url);
                        }
                    }
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::select_urls_one_level_deeper;

    #[test]
    fn test_select_urls_one_level_deeper() {
        let urls = vec![
            "https://example.com/level1/level2",
            "https://example.com/level1/level2/level3",
            "https://example.com/different-level1/level2/level3/level4",
        ];
        let base_url = "https://example.com";
        let expected = vec![
            "https://example.com/level1",
            "https://example.com/different-level1",
        ];
        assert_eq!(select_urls_one_level_deeper(urls, base_url), expected);
    }

    #[test]
    fn test_select_urls_one_level_deeper_with_base_url() {
        let urls = vec![
            "https://example.com/different-level1/level2",
            "https://example.com/level1/level2/level3",
            "https://example.com/level1/different-level2/level3/level4",
        ];
        let base_url = "https://example.com/level1";
        let expected = vec![
            "https://example.com/level1/level2",
            "https://example.com/level1/different-level2",
        ];
        assert_eq!(select_urls_one_level_deeper(urls, base_url), expected);
    }

    #[test]
    fn test_select_urls_one_level_deeper_empty_input() {
        let urls: Vec<&str> = vec![];
        let base_url = "https://example.com";
        let expected: Vec<String> = vec![];
        assert_eq!(select_urls_one_level_deeper(urls, base_url), expected);
    }

    #[test]
    fn test_select_urls_one_level_deeper_invalid_base_url() {
        let urls = vec!["https://example.com/page1"];
        let base_url = "invalid-url";
        let expected: Vec<String> = vec![];
        assert_eq!(select_urls_one_level_deeper(urls, base_url), expected);
    }

    #[test]
    fn test_select_urls_one_level_deeper_different_domains() {
        let urls = vec![
            "https://example.com/page1/subpage",
            "https://otherdomain.com/page1/subpage",
            "https://subdomain.example.com/page1/subpage",
        ];
        let base_url = "https://example.com";
        let expected = vec![
            "https://example.com/page1",
            "https://subdomain.example.com/page1",
        ];
        assert_eq!(select_urls_one_level_deeper(urls, base_url), expected);
    }

    #[test]
    fn test_select_urls_one_level_deeper_no_deeper_urls() {
        let urls = vec!["https://example.com", "https://example.com/"];
        let base_url = "https://example.com";
        let expected: Vec<String> = vec![];
        assert_eq!(select_urls_one_level_deeper(urls, base_url), expected);
    }

    #[test]
    fn test_select_urls_one_level_deeper_root_base() {
        let urls = vec![
            "https://example.com/a/b/c",
            "https://example.com/x/y/z",
            "https://example.com/a/different",
            "https://example.com/single",
        ];
        let base_url = "https://example.com";
        let expected = vec![
            "https://example.com/a",
            "https://example.com/x",
            "https://example.com/single",
        ];
        assert_eq!(select_urls_one_level_deeper(urls, base_url), expected);
    }

    #[test]
    fn test_select_urls_one_level_deeper_with_query_params() {
        let urls = vec![
            "https://example.com/page1/subpage?param=value",
            "https://example.com/page2/subpage#fragment",
        ];
        let base_url = "https://example.com";
        let expected = vec!["https://example.com/page1", "https://example.com/page2"];
        assert_eq!(select_urls_one_level_deeper(urls, base_url), expected);
    }

    #[test]
    fn test_select_urls_one_level_deeper_deduplication() {
        let urls = vec![
            "https://example.com/page1/sub1",
            "https://example.com/page1/sub2",
            "https://example.com/page1/sub3",
            "https://example.com/page2/sub1",
        ];
        let base_url = "https://example.com";
        let expected = vec!["https://example.com/page1", "https://example.com/page2"];
        assert_eq!(select_urls_one_level_deeper(urls, base_url), expected);
    }

    #[test]
    fn test_select_urls_one_level_deeper_trailing_slashes() {
        let urls = vec![
            "https://example.com/page1/subpage/",
            "https://example.com/page2/subpage",
        ];
        let base_url = "https://example.com/";
        let expected = vec!["https://example.com/page1", "https://example.com/page2"];
        assert_eq!(select_urls_one_level_deeper(urls, base_url), expected);
    }

    #[test]
    fn test_select_urls_one_level_deeper_mixed_schemes() {
        let urls = vec![
            "https://example.com/page1/subpage",
            "http://example.com/page2/subpage",
        ];
        let base_url = "https://example.com";
        let expected = vec!["https://example.com/page1", "http://example.com/page2"];
        assert_eq!(select_urls_one_level_deeper(urls, base_url), expected);
    }

    #[test]
    fn test_select_urls_one_level_deeper_exact_match() {
        let urls = vec![
            "https://example.com/level1",        // Exact match with base
            "https://example.com/level1/level2", // One level deeper
        ];
        let base_url = "https://example.com/level1";
        let expected = vec!["https://example.com/level1/level2"];
        assert_eq!(select_urls_one_level_deeper(urls, base_url), expected);
    }
}
