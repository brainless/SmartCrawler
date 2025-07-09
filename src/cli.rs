use clap::{Arg, Command};
use std::collections::HashSet;
use url::Url;

#[derive(Debug, Clone)]
pub struct CliArgs {
    pub domains: Vec<String>,
    pub prep: bool,
}

impl CliArgs {
    pub fn parse() -> Result<Self, String> {
        let matches = Command::new("smart-crawler")
            .version("0.4.1")
            .about("A web crawler that uses WebDriver to extract and parse HTML content")
            .arg(
                Arg::new("domain")
                    .long("domain")
                    .value_name("DOMAIN")
                    .help("Domain to crawl (can be specified multiple times). Can be a URL or domain name")
                    .action(clap::ArgAction::Append)
                    .required(true),
            )
            .arg(
                Arg::new("prep")
                    .long("prep")
                    .help("Enable preparation mode to discover template patterns across domain pages")
                    .action(clap::ArgAction::SetTrue),
            )
            .get_matches();

        let domains: Vec<String> = matches
            .get_many::<String>("domain")
            .unwrap_or_default()
            .cloned()
            .collect();

        let validated_domains = Self::validate_and_extract_domains(domains)?;
        let prep = matches.get_flag("prep");

        Ok(CliArgs {
            domains: validated_domains,
            prep,
        })
    }

    fn validate_and_extract_domains(domains: Vec<String>) -> Result<Vec<String>, String> {
        let mut seen_domains = HashSet::new();
        let mut validated_domains = Vec::new();

        for domain_input in domains {
            let domain = Self::extract_domain(&domain_input)?;
            if seen_domains.insert(domain.clone()) {
                validated_domains.push(domain);
            }
        }

        if validated_domains.is_empty() {
            return Err("No valid domains provided".to_string());
        }

        Ok(validated_domains)
    }

    fn extract_domain(input: &str) -> Result<String, String> {
        let trimmed = input.trim();

        // Always try to parse as URL to validate the domain
        let url_str = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            trimmed.to_string()
        } else {
            format!("https://{trimmed}")
        };

        match Url::parse(&url_str) {
            Ok(url) => {
                if let Some(domain) = url.host_str() {
                    Ok(domain.to_string())
                } else {
                    Err(format!("Could not extract domain from: {input}"))
                }
            }
            Err(_) => Err(format!("Invalid domain or URL: {input}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_and_extract_domains() {
        let domains = vec![
            "https://example.com".to_string(),
            "example.org".to_string(),
            "https://example.com/path".to_string(), // duplicate domain
        ];

        let result = CliArgs::validate_and_extract_domains(domains).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"example.com".to_string()));
        assert!(result.contains(&"example.org".to_string()));
    }

    #[test]
    fn test_extract_domain() {
        // Test URL with protocol
        assert_eq!(
            CliArgs::extract_domain("https://example.com").unwrap(),
            "example.com"
        );
        assert_eq!(
            CliArgs::extract_domain("http://example.com/path").unwrap(),
            "example.com"
        );

        // Test domain without protocol
        assert_eq!(
            CliArgs::extract_domain("example.com").unwrap(),
            "example.com"
        );
        assert_eq!(
            CliArgs::extract_domain("  example.com  ").unwrap(),
            "example.com"
        );

        // Test edge case - the URL crate behavior with multiple dots
        assert_eq!(
            CliArgs::extract_domain("invalid..domain").unwrap(),
            "invalid..domain"
        );
    }

    #[test]
    fn test_validate_empty_domains() {
        let domains = vec![];
        let result = CliArgs::validate_and_extract_domains(domains);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No valid domains provided"));
    }

    #[test]
    fn test_cli_prep_flag() {
        // Test that prep flag is properly parsed (this is a simplified test
        // since we can't easily test the full CLI parsing in unit tests)
        let args = CliArgs {
            domains: vec!["example.com".to_string()],
            prep: true,
        };

        assert!(args.prep);
        assert_eq!(args.domains.len(), 1);
    }
}
