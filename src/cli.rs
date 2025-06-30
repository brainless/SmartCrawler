use crate::url_ranking::UrlRankingConfig;
use clap::{Arg, Command};
use std::env;
use std::sync::{Arc, Mutex};
use url::Url;

#[derive(Debug, Clone)]
pub struct CrawlerConfig {
    pub objective: String,
    pub domains: Arc<Mutex<Vec<String>>>,
    pub max_urls_per_domain: usize,
    pub delay_ms: u64,
    pub output_file: Option<String>,
    pub verbose: bool,
    pub url_ranking_config: UrlRankingConfig,
    pub enable_keyword_filtering: bool,
}

#[derive(Debug, Clone)]
pub struct CleanHtmlConfig {
    pub input_file: String,
    pub output_file: String,
    pub verbose: bool,
}

#[derive(Debug, Clone)]
pub enum AppMode {
    Crawl(CrawlerConfig),
    CleanHtml(CleanHtmlConfig),
}

/// Extract domain from a URL string, handling both full URLs and plain domains
/// Removes trailing slashes and returns just the domain part
/// Examples:
/// - "https://example.com/" -> "example.com"
/// - "http://example.com" -> "example.com"  
/// - "example.com" -> "example.com"
fn extract_domain_from_url(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    
    // If it doesn't contain ://, assume it's already a domain
    if !trimmed.contains("://") {
        let domain = trimmed.trim_end_matches('/').to_string();
        if domain.is_empty() {
            return Err("Domain cannot be empty".to_string());
        }
        return Ok(domain);
    }
    
    // Parse as URL
    match Url::parse(trimmed) {
        Ok(url) => {
            if let Some(host) = url.host_str() {
                Ok(host.to_string())
            } else {
                Err(format!("No valid domain found in URL: {}", trimmed))
            }
        }
        Err(e) => Err(format!("Failed to parse URL '{}': {}", trimmed, e)),
    }
}

impl CrawlerConfig {
    pub fn add_domain(&self, domain: String) {
        let mut domains = self.domains.lock().unwrap();
        if !domains.contains(&domain) {
            domains.push(domain);
        }
    }
    pub fn from_args() -> AppMode {
        let matches = Command::new("smart-crawler")
            .version("1.0.0")
            .author("Smart Crawler")
            .about("Intelligent web crawler that uses Claude AI to select relevant URLs")
            .subcommand_required(false)
            .arg(
                Arg::new("objective")
                    .short('o')
                    .long("objective")
                    .value_name("OBJECTIVE")
                    .help("The crawling objective - what information to look for")
                    .required_unless_present("clean-html")
            )
            .arg(
                Arg::new("domains")
                    .short('d')
                    .long("domains")
                    .value_name("DOMAINS")
                    .help("Comma-separated list of domains to crawl")
                    .required_unless_present("clean-html")
            )
            .arg(
                Arg::new("max-urls")
                    .short('m')
                    .long("max-urls")
                    .value_name("NUMBER")
                    .help("Maximum URLs to crawl per domain")
                    .default_value("10")
            )
            .arg(
                Arg::new("delay")
                    .long("delay")
                    .value_name("MILLISECONDS")
                    .help("Delay between requests in milliseconds")
                    .default_value("1000")
            )
            .arg(
                Arg::new("output")
                    .short('O')
                    .long("output")
                    .value_name("FILE")
                    .help("Output file for results (JSON format)")
            )
            .arg(
                Arg::new("clean-html")
                    .long("clean-html")
                    .value_names(["INPUT_FILE", "OUTPUT_FILE"])
                    .num_args(2)
                    .help("Clean HTML file by removing unwanted elements and attributes. Usage: --clean-html <input.html> <output.html>")
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("Enable verbose logging")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("disable-keyword-filtering")
                    .long("disable-keyword-filtering")
                    .help("Disable keyword-based URL pre-filtering")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("candidate-multiplier")
                    .long("candidate-multiplier")
                    .value_name("NUMBER")
                    .help("Multiplier for candidate URLs sent to LLM (candidate_count = max_urls * multiplier)")
                    .default_value("3")
            )
            .get_matches();

        let verbose = matches.get_flag("verbose");

        // Check if clean-html mode is requested
        if let Some(clean_html_args) = matches.get_many::<String>("clean-html") {
            let args: Vec<&String> = clean_html_args.collect();
            if args.len() != 2 {
                eprintln!(
                    "Error: --clean-html requires exactly 2 arguments: <input_file> <output_file>"
                );
                std::process::exit(1);
            }
            return AppMode::CleanHtml(CleanHtmlConfig {
                input_file: args[0].clone(),
                output_file: args[1].clone(),
                verbose,
            });
        }

        // Default crawl mode
        let objective = matches.get_one::<String>("objective").unwrap().clone();

        let domains_str = matches.get_one::<String>("domains").unwrap();
        let domains: Result<Vec<String>, String> = domains_str
            .split(',')
            .map(extract_domain_from_url)
            .collect();
        
        let domains = match domains {
            Ok(domains) => domains,
            Err(e) => {
                eprintln!("Error parsing domains: {}", e);
                std::process::exit(1);
            }
        };

        let max_urls_per_domain = matches
            .get_one::<String>("max-urls")
            .unwrap()
            .parse()
            .unwrap_or(10);

        let delay_ms = matches
            .get_one::<String>("delay")
            .unwrap()
            .parse()
            .unwrap_or(1000);

        let output_file = matches.get_one::<String>("output").cloned();

        let enable_keyword_filtering = !matches.get_flag("disable-keyword-filtering");

        let candidate_multiplier = matches
            .get_one::<String>("candidate-multiplier")
            .unwrap()
            .parse()
            .unwrap_or(3);

        let url_ranking_config = UrlRankingConfig {
            candidate_multiplier,
            ..UrlRankingConfig::default()
        };

        AppMode::Crawl(CrawlerConfig {
            objective,
            domains: Arc::new(Mutex::new(domains)),
            max_urls_per_domain,
            delay_ms,
            output_file,
            verbose,
            url_ranking_config,
            enable_keyword_filtering,
        })
    }
}

impl CleanHtmlConfig {
    pub fn validate(&self) -> Result<(), String> {
        use std::path::Path;

        if !Path::new(&self.input_file).exists() {
            return Err(format!("Input file does not exist: {}", self.input_file));
        }

        // Note: We allow output files in any directory - the file operation will fail if directory doesn't exist

        Ok(())
    }
}

impl CrawlerConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.objective.trim().is_empty() {
            return Err("Objective cannot be empty".to_string());
        }

        let domains = self.domains.lock().unwrap();
        if domains.is_empty() {
            return Err("At least one domain must be specified".to_string());
        }

        for domain in domains.iter() {
            if domain.trim().is_empty() {
                return Err("Domain names cannot be empty".to_string());
            }
        }

        if self.max_urls_per_domain == 0 {
            return Err("Max URLs per domain must be greater than 0".to_string());
        }

        // Check if ANTHROPIC_API_KEY is set
        if env::var("ANTHROPIC_API_KEY").is_err() {
            return Err("ANTHROPIC_API_KEY environment variable must be set".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain_from_url() {
        // Test full URLs with https
        assert_eq!(
            extract_domain_from_url("https://example.com").unwrap(),
            "example.com"
        );
        assert_eq!(
            extract_domain_from_url("https://example.com/").unwrap(),
            "example.com"
        );
        assert_eq!(
            extract_domain_from_url("https://www.example.com/path?query=value").unwrap(),
            "www.example.com"
        );

        // Test full URLs with http  
        assert_eq!(
            extract_domain_from_url("http://example.com").unwrap(),
            "example.com"
        );
        assert_eq!(
            extract_domain_from_url("http://example.com/").unwrap(),
            "example.com"
        );

        // Test plain domains (existing behavior)
        assert_eq!(
            extract_domain_from_url("example.com").unwrap(),
            "example.com"
        );
        assert_eq!(
            extract_domain_from_url("www.example.com").unwrap(),
            "www.example.com"
        );
        assert_eq!(
            extract_domain_from_url("subdomain.example.com").unwrap(),
            "subdomain.example.com"
        );

        // Test trimming trailing slashes from plain domains
        assert_eq!(
            extract_domain_from_url("example.com/").unwrap(),
            "example.com"
        );
        assert_eq!(
            extract_domain_from_url("example.com//").unwrap(),
            "example.com"
        );

        // Test whitespace handling
        assert_eq!(
            extract_domain_from_url("  https://example.com  ").unwrap(),
            "example.com"
        );
        assert_eq!(
            extract_domain_from_url("  example.com  ").unwrap(),
            "example.com"
        );

        // Test error cases
        assert!(extract_domain_from_url("").is_err());
        assert!(extract_domain_from_url("   ").is_err());
        assert!(extract_domain_from_url("/").is_err());
        assert!(extract_domain_from_url("://example.com").is_err());
        assert!(extract_domain_from_url("https://").is_err());
    }
}
