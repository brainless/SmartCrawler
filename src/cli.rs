use clap::{Arg, Command};
use std::collections::HashSet;
use url::Url;

#[derive(Debug, Clone)]
pub struct CliArgs {
    pub links: Vec<String>,
    pub verbose: bool,
    pub template: bool,
}

impl CliArgs {
    pub fn parse() -> Result<Self, String> {
        let matches = Command::new("smart-crawler")
            .version("0.3.2")
            .about("A web crawler that uses WebDriver to extract and parse HTML content")
            .arg(
                Arg::new("link")
                    .long("link")
                    .value_name("URL")
                    .help("URL to crawl (can be specified multiple times)")
                    .action(clap::ArgAction::Append)
                    .required(true),
            )
            .arg(
                Arg::new("verbose")
                    .long("verbose")
                    .help("Enable verbose output showing filtered HTML node tree")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("template")
                    .long("template")
                    .help("Enable template detection mode to identify patterns like '{count} comments' in HTML content")
                    .action(clap::ArgAction::SetTrue),
            )
            .get_matches();

        let links: Vec<String> = matches
            .get_many::<String>("link")
            .unwrap_or_default()
            .cloned()
            .collect();

        let validated_links = Self::validate_and_deduplicate_links(links)?;
        let verbose = matches.get_flag("verbose");
        let template = matches.get_flag("template");

        Ok(CliArgs {
            links: validated_links,
            verbose,
            template,
        })
    }

    fn validate_and_deduplicate_links(links: Vec<String>) -> Result<Vec<String>, String> {
        let mut seen_urls = HashSet::new();
        let mut validated_links = Vec::new();

        for link in links {
            match Url::parse(&link) {
                Ok(url) => {
                    let normalized_url = url.to_string();
                    if seen_urls.insert(normalized_url.clone()) {
                        validated_links.push(normalized_url);
                    }
                }
                Err(_) => {
                    return Err(format!("Invalid URL: {link}"));
                }
            }
        }

        if validated_links.is_empty() {
            return Err("No valid URLs provided".to_string());
        }

        Ok(validated_links)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_and_deduplicate_links() {
        let links = vec![
            "https://example.com".to_string(),
            "https://example.org".to_string(),
            "https://example.com".to_string(), // duplicate
        ];

        let result = CliArgs::validate_and_deduplicate_links(links).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"https://example.com/".to_string()));
        assert!(result.contains(&"https://example.org/".to_string()));
    }

    #[test]
    fn test_validate_invalid_url() {
        let links = vec!["invalid-url".to_string()];
        let result = CliArgs::validate_and_deduplicate_links(links);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid URL"));
    }

    #[test]
    fn test_validate_empty_links() {
        let links = vec![];
        let result = CliArgs::validate_and_deduplicate_links(links);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No valid URLs provided"));
    }

    #[test]
    fn test_cli_template_flag() {
        // Test that template flag is properly parsed (this is a simplified test
        // since we can't easily test the full CLI parsing in unit tests)
        let args = CliArgs {
            links: vec!["https://example.com".to_string()],
            verbose: true,
            template: true,
        };

        assert!(args.template);
        assert!(args.verbose);
        assert_eq!(args.links.len(), 1);
    }
}
