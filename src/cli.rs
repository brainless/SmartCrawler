use clap::{Arg, Command};
use std::env;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct CrawlerConfig {
    pub objective: String,
    pub domains: Arc<Mutex<Vec<String>>>,
    pub max_urls_per_domain: usize,
    pub delay_ms: u64,
    pub output_file: Option<String>,
    pub verbose: bool,
}

impl CrawlerConfig {
    pub fn add_domain(&self, domain: String) {
        let mut domains = self.domains.lock().unwrap();
        if !domains.contains(&domain) {
            domains.push(domain);
        }
    }
    pub fn from_args() -> Self {
        let matches = Command::new("smart-crawler")
            .version("1.0.0")
            .author("Smart Crawler")
            .about("Intelligent web crawler that uses Claude AI to select relevant URLs")
            .arg(
                Arg::new("objective")
                    .short('o')
                    .long("objective")
                    .value_name("OBJECTIVE")
                    .help("The crawling objective - what information to look for")
                    .required(true)
            )
            .arg(
                Arg::new("domains")
                    .short('d')
                    .long("domains")
                    .value_name("DOMAINS")
                    .help("Comma-separated list of domains to crawl")
                    .required(true)
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
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("Enable verbose logging")
                    .action(clap::ArgAction::SetTrue)
            )
            .get_matches();

        let objective = matches.get_one::<String>("objective").unwrap().clone();
        
        let domains_str = matches.get_one::<String>("domains").unwrap();
        let domains: Vec<String> = domains_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

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
        let verbose = matches.get_flag("verbose");

        CrawlerConfig {
            objective,
            domains: Arc::new(Mutex::new(domains)),
            max_urls_per_domain,
            delay_ms,
            output_file,
            verbose,
        }
    }

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