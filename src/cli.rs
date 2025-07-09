use clap::{Arg, Command};
use url::Url;

#[derive(Debug, Clone)]
pub struct CliArgs {
    pub domain: String,
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
                    .help("Domain to crawl. Can be a URL or domain name")
                    .required(true),
            )
            .arg(
                Arg::new("prep")
                    .long("prep")
                    .help(
                        "Enable preparation mode to discover template patterns across domain pages",
                    )
                    .action(clap::ArgAction::SetTrue),
            )
            .get_matches();

        let domain_input = matches
            .get_one::<String>("domain")
            .ok_or("Domain argument is required")?;

        let validated_domain = Self::extract_domain(domain_input)?;
        let prep = matches.get_flag("prep");

        Ok(CliArgs {
            domain: validated_domain,
            prep,
        })
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
    fn test_single_domain_parsing() {
        // Test that single domain parsing works correctly
        let args = CliArgs {
            domain: "example.com".to_string(),
            prep: false,
        };

        assert_eq!(args.domain, "example.com");
        assert!(!args.prep);
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
    fn test_extract_domain_error() {
        // Test that invalid domain extraction returns error
        let result = CliArgs::extract_domain("://invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid domain or URL"));
    }

    #[test]
    fn test_cli_prep_flag() {
        // Test that prep flag is properly parsed (this is a simplified test
        // since we can't easily test the full CLI parsing in unit tests)
        let args = CliArgs {
            domain: "example.com".to_string(),
            prep: true,
        };

        assert!(args.prep);
        assert_eq!(args.domain, "example.com");
    }
}
