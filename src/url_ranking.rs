/// URL ranking and scoring utilities for keyword-based URL filtering
use url::Url;

/// Configuration for URL ranking algorithm
#[derive(Debug, Clone)]
pub struct UrlRankingConfig {
    /// Maximum number of candidate URLs to send to LLM (multiplier * max_urls)
    pub candidate_multiplier: usize,
    /// Weight for keyword matches in URL path
    pub path_weight: f32,
    /// Weight for keyword matches in query parameters
    pub query_weight: f32,
    /// Weight for URL depth (shorter paths score higher)
    pub depth_weight: f32,
    /// Bonus score for exact keyword matches
    pub exact_match_bonus: f32,
    /// Bonus score for partial keyword matches
    pub partial_match_bonus: f32,
}

impl Default for UrlRankingConfig {
    fn default() -> Self {
        Self {
            candidate_multiplier: 3,
            path_weight: 1.0,
            query_weight: 0.8,
            depth_weight: 0.3,
            exact_match_bonus: 2.0,
            partial_match_bonus: 1.0,
        }
    }
}

/// Score and rank URLs based on keyword relevance
pub struct UrlRanker {
    config: UrlRankingConfig,
}

/// URL with computed relevance score
#[derive(Debug, Clone)]
pub struct ScoredUrl {
    pub url: String,
    pub score: f32,
    pub path: String,
    pub query: Option<String>,
    pub depth: usize,
}

impl UrlRanker {
    pub fn new(config: UrlRankingConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(UrlRankingConfig::default())
    }

    /// Score and rank URLs based on keyword relevance
    pub fn rank_urls(
        &self,
        urls: &[String],
        keywords: &[String],
        max_urls: usize,
    ) -> Vec<String> {
        let candidate_limit = max_urls * self.config.candidate_multiplier;

        // Score all URLs
        let mut scored_urls: Vec<ScoredUrl> = urls
            .iter()
            .filter_map(|url| self.score_url(url, keywords))
            .collect();

        // Sort by score (highest first)
        scored_urls.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Take top candidates
        let top_urls: Vec<String> = scored_urls
            .into_iter()
            .take(candidate_limit)
            .map(|scored| scored.url)
            .collect();

        tracing::info!(
            "Ranked {} URLs, selected top {} candidates for LLM selection",
            urls.len(),
            top_urls.len()
        );

        if !top_urls.is_empty() {
            tracing::debug!("Top ranked URLs: {:?}", &top_urls[..top_urls.len().min(5)]);
        }

        top_urls
    }

    /// Score a single URL based on keyword relevance
    fn score_url(&self, url: &str, keywords: &[String]) -> Option<ScoredUrl> {
        let parsed_url = Url::parse(url).ok()?;
        let path = parsed_url.path().to_lowercase();
        let query = parsed_url.query().map(|q| q.to_lowercase());
        let depth = path.split('/').filter(|segment| !segment.is_empty()).count();

        let mut score = 0.0;

        // Score based on keyword matches in path
        for keyword in keywords {
            let keyword_lower = keyword.to_lowercase();
            
            // Path scoring
            if path.contains(&keyword_lower) {
                if path.split('/').any(|segment| segment == keyword_lower) {
                    // Exact match in path segment
                    score += self.config.exact_match_bonus * self.config.path_weight;
                } else {
                    // Partial match in path
                    score += self.config.partial_match_bonus * self.config.path_weight;
                }
            }

            // Query parameter scoring
            if let Some(ref query_str) = query {
                if query_str.contains(&keyword_lower) {
                    if query_str.split('&').any(|param| {
                        param.split('=').any(|part| part == keyword_lower)
                    }) {
                        // Exact match in query parameter
                        score += self.config.exact_match_bonus * self.config.query_weight;
                    } else {
                        // Partial match in query
                        score += self.config.partial_match_bonus * self.config.query_weight;
                    }
                }
            }
        }

        // Depth penalty (prefer shorter paths)
        let depth_penalty = depth as f32 * self.config.depth_weight;
        score -= depth_penalty;

        // Ensure non-negative score
        score = score.max(0.0);

        Some(ScoredUrl {
            url: url.to_string(),
            score,
            path: path.clone(),
            query,
            depth,
        })
    }

    /// Get statistics about URL scoring
    pub fn get_scoring_stats(
        &self,
        urls: &[String],
        keywords: &[String],
    ) -> UrlScoringStats {
        let scored_urls: Vec<ScoredUrl> = urls
            .iter()
            .filter_map(|url| self.score_url(url, keywords))
            .collect();

        let scores: Vec<f32> = scored_urls.iter().map(|u| u.score).collect();
        let total_urls = scored_urls.len();
        let scored_urls_count = scores.iter().filter(|&&s| s > 0.0).count();

        let (min_score, max_score, avg_score) = if scores.is_empty() {
            (0.0, 0.0, 0.0)
        } else {
            let min = scores.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            let sum: f32 = scores.iter().sum();
            let avg = sum / scores.len() as f32;
            (min, max, avg)
        };

        UrlScoringStats {
            total_urls,
            scored_urls_count,
            min_score,
            max_score,
            avg_score,
        }
    }
}

/// Statistics about URL scoring results
#[derive(Debug)]
pub struct UrlScoringStats {
    pub total_urls: usize,
    pub scored_urls_count: usize,
    pub min_score: f32,
    pub max_score: f32,
    pub avg_score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_scoring_exact_match() {
        let ranker = UrlRanker::with_default_config();
        let keywords = vec!["pricing".to_string(), "plans".to_string()];
        
        let scored = ranker.score_url("https://example.com/pricing", &keywords).unwrap();
        assert!(scored.score > 0.0);
        assert_eq!(scored.path, "/pricing");
    }

    #[test]
    fn test_url_scoring_partial_match() {
        let ranker = UrlRanker::with_default_config();
        let keywords = vec!["price".to_string()];
        
        let scored = ranker.score_url("https://example.com/price-info", &keywords).unwrap();
        assert!(scored.score > 0.0);
    }

    #[test]
    fn test_url_scoring_query_params() {
        let ranker = UrlRanker::with_default_config();
        let keywords = vec!["pricing".to_string()];
        
        let scored = ranker.score_url("https://example.com/info?category=pricing", &keywords).unwrap();
        assert!(scored.score > 0.0);
    }

    #[test]
    fn test_url_ranking_order() {
        let ranker = UrlRanker::with_default_config();
        let urls = vec![
            "https://example.com/about".to_string(),
            "https://example.com/pricing".to_string(),
            "https://example.com/pricing/plans".to_string(),
            "https://example.com/contact".to_string(),
        ];
        let keywords = vec!["pricing".to_string(), "plans".to_string()];
        
        let ranked = ranker.rank_urls(&urls, &keywords, 2);
        
        // Should prioritize URLs with keyword matches
        assert!(ranked.contains(&"https://example.com/pricing".to_string()));
        assert!(ranked.contains(&"https://example.com/pricing/plans".to_string()));
    }

    #[test]
    fn test_url_ranking_candidate_limit() {
        let ranker = UrlRanker::with_default_config();
        let urls: Vec<String> = (1..=20)
            .map(|i| format!("https://example.com/page{}", i))
            .collect();
        let keywords = vec!["test".to_string()];
        
        let ranked = ranker.rank_urls(&urls, &keywords, 2);
        
        // Should limit to candidate_multiplier * max_urls = 3 * 2 = 6
        assert_eq!(ranked.len(), 6);
    }

    #[test]
    fn test_scoring_stats() {
        let ranker = UrlRanker::with_default_config();
        let urls = vec![
            "https://example.com/pricing".to_string(),
            "https://example.com/about".to_string(),
        ];
        let keywords = vec!["pricing".to_string()];
        
        let stats = ranker.get_scoring_stats(&urls, &keywords);
        
        assert_eq!(stats.total_urls, 2);
        assert!(stats.max_score > stats.min_score);
    }
}