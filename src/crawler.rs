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
        tracing::info!("Starting crawl for domain: {}", domain);

        let browser = Browser::new().await?;
        let mut scraped_content = Vec::new();
        let mut analysis = Vec::new();
        let mut extracted_entities = Vec::new();
        let mut objective_met = false;
        let mut selected_urls = Vec::new();

        let root_url = format!("https://{domain}");

        // Step 1: Get sitemap URLs
        tracing::info!("Discovering sitemap for: {}", domain);
        let sitemap_urls = self.sitemap_parser.get_all_urls(domain).await?;
        tracing::info!(
            "Found {} URLs in sitemap for {}",
            sitemap_urls.len(),
            domain
        );

        // Step 2: Always analyze homepage first
        tracing::info!("Analyzing homepage as first URL: {}", root_url);
        let mut homepage_analyzed = false;
        let mut homepage_content: Option<ScrapedWebPage> = None;

        match browser.scrape_url(&root_url).await {
            Ok(web_page) => {
                tracing::info!("Successfully scraped homepage: {}", web_page.url);

                // Analyze homepage content
                match self.analyze_page_content(&web_page).await {
                    Ok((page_analysis, entities, met)) => {
                        analysis.extend(page_analysis);
                        extracted_entities.extend(entities);
                        objective_met = met;
                        homepage_analyzed = true;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to analyze homepage content: {}", e);
                        analysis.push(format!("Homepage analysis failed: {e}"));
                    }
                }

                homepage_content = Some(web_page.clone());
                scraped_content.push(web_page);
                selected_urls.push(root_url.clone());

                // Mark homepage as scraped to avoid duplicates
                self.mark_url_as_scraped(domain, &root_url);
            }
            Err(e) => {
                tracing::warn!("Failed to scrape homepage {}: {}", root_url, e);
                analysis.push(format!("Homepage scraping failed: {e}"));
            }
        }

        // Step 3: If sitemap is empty and homepage was analyzed, we're done
        if sitemap_urls.is_empty() && homepage_analyzed {
            tracing::info!(
                "No sitemap found, homepage analysis complete for {}",
                domain
            );
            browser.close().await?;
            return Ok(CrawlResult {
                domain: domain.to_string(),
                objective: self.config.objective.clone(),
                selected_urls,
                scraped_content,
                analysis,
                extracted_entities,
            });
        }

        // Step 4: Discover additional URLs for further analysis
        let mut urls_to_analyze: Vec<String> = sitemap_urls
            .iter()
            .map(|u| u.as_ref().to_string())
            .collect();

        // Get additional URLs from homepage if it was successfully scraped
        if let Some(homepage) = &homepage_content {
            let mut discovered_urls = homepage.links.clone();

            // Filter to only include URLs from the same domain
            discovered_urls.retain(|url| {
                if let Ok(parsed_url) = url::Url::parse(url) {
                    if let Some(url_domain) = parsed_url.host_str() {
                        return url_domain == domain || url_domain.ends_with(&format!(".{domain}"));
                    }
                }
                false
            });

            // Remove duplicates and exclude homepage itself
            let unique_urls: std::collections::HashSet<String> =
                discovered_urls.into_iter().collect();
            let homepage_links: Vec<String> = unique_urls
                .into_iter()
                .filter(|url| url != &root_url) // Exclude homepage since we already analyzed it
                .collect();

            tracing::info!(
                "Discovered {} additional URLs from homepage of {}",
                homepage_links.len(),
                domain
            );
            urls_to_analyze.extend(homepage_links);
        }

        // Only retain URLs that are one level deeper
        let urls_one_level_deeper =
            select_urls_one_level_deeper(urls_to_analyze.clone(), format!("https://{domain}"));

        // Step 5: Add URLs that match objective keywords (improvement from issue #19)
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

        // Step 6: URL Selection for remaining URLs (with optional keyword-based pre-filtering)
        let mut additional_selected_urls = if self.config.enable_keyword_filtering {
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
            if urls_to_analyze.is_empty() {
                Vec::new()
            } else {
                self.llm_client
                    .select_urls(
                        &self.config.objective,
                        &urls_to_analyze,
                        domain,
                        self.config.max_urls_per_domain,
                    )
                    .await?
            }
        };

        // Step 7: Filter out already scraped URLs and track unique URLs (using path+query only)
        {
            let mut scraped_urls = self.urls_scraped.lock().unwrap();
            let domain_paths = scraped_urls.entry(domain.to_string()).or_default();

            additional_selected_urls.retain(|url: &String| {
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

        // Add additional URLs to our selected list
        selected_urls.extend(additional_selected_urls.clone());

        tracing::info!(
            "Selected {} additional URLs for {} (after deduplication, {} total including homepage)",
            additional_selected_urls.len(),
            domain,
            selected_urls.len()
        );

        // If no additional URLs were selected, we're done (only homepage was analyzed)
        if additional_selected_urls.is_empty() {
            tracing::info!(
                "No additional URLs selected for {}, analysis complete",
                domain
            );
            browser.close().await?;
            return Ok(CrawlResult {
                domain: domain.to_string(),
                objective: self.config.objective.clone(),
                selected_urls,
                scraped_content,
                analysis,
                extracted_entities,
            });
        }

        // Step 8: Scrape and analyze additional selected URLs
        tracing::info!(
            "Scraping {} additional selected URLs for {}",
            additional_selected_urls.len(),
            domain
        );

        // Analyze additional URLs sequentially
        for url in &additional_selected_urls {
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

                    // Analyze this page's content
                    match self.analyze_page_content(&web_page).await {
                        Ok((page_analysis, entities, met)) => {
                            analysis.extend(page_analysis);
                            extracted_entities.extend(entities);
                            if met {
                                objective_met = true;
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to analyze content for {}: {}", web_page.url, e);
                            analysis.push(format!("URL: {}\nAnalysis failed: {e}", web_page.url));
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

    /// Helper method to analyze a single page's content and return structured results
    async fn analyze_page_content(
        &self,
        web_page: &ScrapedWebPage,
    ) -> Result<(Vec<String>, Vec<EntityExtractionResult>, bool), CrawlerError> {
        let mut page_analysis = Vec::new();
        let mut page_entities = Vec::new();

        // Extract structured entities from the content
        let objective_met = match self
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

                page_analysis.push(format!(
                    "URL: {}\nExtracted {} entities with {:.1}% confidence\nAnalysis: {}",
                    web_page.url,
                    entity_count,
                    confidence * 100.0,
                    entity_result.raw_analysis
                ));

                page_entities.push(entity_result);
                entity_count > 0 && confidence > 0.5
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
                            let results_count =
                                llm_response.results.as_ref().map(|r| r.len()).unwrap_or(0);
                            page_analysis.push(format!(
                                "URL: {}\nObjective Met: {}\nFound {} results\nAnalysis: {}",
                                web_page.url,
                                llm_response.is_objective_met,
                                results_count,
                                llm_response
                                    .analysis
                                    .unwrap_or_else(|| "No analysis provided".to_string())
                            ));
                            true
                        } else {
                            page_analysis.push(format!(
                                "URL: {}\nObjective Not Met\nAnalysis: {}",
                                web_page.url,
                                llm_response
                                    .analysis
                                    .unwrap_or_else(|| { "No analysis provided".to_string() })
                            ));
                            false
                        }
                    }
                    Err(_) => false,
                }
            }
        };

        Ok((page_analysis, page_entities, objective_met))
    }

    /// Helper method to mark a URL as scraped in the tracking system
    fn mark_url_as_scraped(&self, domain: &str, url: &str) {
        if let Some(path_query) = extract_path_query(url) {
            let mut scraped_urls = self.urls_scraped.lock().unwrap();
            let domain_paths = scraped_urls.entry(domain.to_string()).or_default();
            domain_paths.insert(path_query);
        }
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

    #[test]
    fn test_extract_path_query_with_query() {
        let url = "https://example.com/path/to/page?param=value&other=123";
        let result = super::extract_path_query(url);
        assert_eq!(
            result,
            Some("/path/to/page?param=value&other=123".to_string())
        );
    }

    #[test]
    fn test_extract_path_query_without_query() {
        let url = "https://example.com/path/to/page";
        let result = super::extract_path_query(url);
        assert_eq!(result, Some("/path/to/page".to_string()));
    }

    #[test]
    fn test_extract_path_query_root_url() {
        let url = "https://example.com/";
        let result = super::extract_path_query(url);
        assert_eq!(result, Some("/".to_string()));
    }

    #[test]
    fn test_extract_path_query_invalid_url() {
        let url = "not-a-valid-url";
        let result = super::extract_path_query(url);
        assert_eq!(result, None);
    }

    #[test]
    fn test_matches_objective_keywords_match() {
        let path_query = "/pricing/plans?category=business";
        let objective = "Find pricing information for business plans";
        assert!(super::matches_objective_keywords(path_query, objective));
    }

    #[test]
    fn test_matches_objective_keywords_no_match() {
        let path_query = "/about/team";
        let objective = "Find pricing information";
        assert!(!super::matches_objective_keywords(path_query, objective));
    }

    #[test]
    fn test_matches_objective_keywords_partial_match() {
        let path_query = "/products/pricing-details";
        let objective = "Find pricing details";
        assert!(super::matches_objective_keywords(path_query, objective));
    }

    #[test]
    fn test_matches_objective_keywords_case_insensitive() {
        let path_query = "/PRICING/PLANS";
        let objective = "find pricing information";
        assert!(super::matches_objective_keywords(path_query, objective));
    }

    #[test]
    fn test_matches_objective_keywords_short_words_filtered() {
        let path_query = "/contact/us";
        let objective = "Find us on the web"; // "us" and "on" are short words that should be filtered
        assert!(!super::matches_objective_keywords(path_query, objective));
    }
}
