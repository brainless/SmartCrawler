use regex::Regex;

pub fn trim_and_clean_text(text: &str) -> String {
    let cleaned = text
        .trim()
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join(" ");

    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(&cleaned, " ").to_string()
}

pub fn extract_domain_from_url(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(|host| host.to_string()))
}

pub fn construct_root_url(domain: &str) -> String {
    let url_string = format!("https://{domain}");
    // Normalize the URL the same way CLI does to ensure consistency
    if let Ok(parsed) = url::Url::parse(&url_string) {
        parsed.to_string()
    } else {
        url_string
    }
}

pub fn is_root_url(url: &str) -> bool {
    if let Ok(parsed) = url::Url::parse(url) {
        let path = parsed.path();
        let query = parsed.query();
        let fragment = parsed.fragment();

        // Root URL has path "/" or empty, no query parameters, and no fragment
        (path == "/" || path.is_empty()) && query.is_none() && fragment.is_none()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_and_clean_text() {
        assert_eq!(trim_and_clean_text("  hello   world  "), "hello world");
        assert_eq!(
            trim_and_clean_text("line1\n  line2  \n\nline3"),
            "line1 line2 line3"
        );
        assert_eq!(trim_and_clean_text(""), "");
        assert_eq!(trim_and_clean_text("   \n  \n  "), "");
    }

    #[test]
    fn test_extract_domain_from_url() {
        assert_eq!(
            extract_domain_from_url("https://example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            extract_domain_from_url("http://subdomain.example.com"),
            Some("subdomain.example.com".to_string())
        );
        assert_eq!(extract_domain_from_url("invalid-url"), None);
    }

    #[test]
    fn test_construct_root_url() {
        assert_eq!(construct_root_url("example.com"), "https://example.com/");
        assert_eq!(
            construct_root_url("subdomain.example.com"),
            "https://subdomain.example.com/"
        );
    }

    #[test]
    fn test_construct_root_url_matches_cli_normalization() {
        // Test that construct_root_url produces URLs that match CLI normalization
        // This prevents duplicate loading when user provides URLs with trailing slashes
        let domain = "news.ycombinator.com";
        let constructed_root = construct_root_url(domain);

        // Simulate what CLI normalization does for user input with trailing slash
        let user_input = "https://news.ycombinator.com/";
        let cli_normalized = url::Url::parse(user_input).unwrap().to_string();

        assert_eq!(constructed_root, cli_normalized);
        assert_eq!(constructed_root, "https://news.ycombinator.com/");
    }

    #[test]
    fn test_is_root_url() {
        assert!(is_root_url("https://example.com"));
        assert!(is_root_url("https://example.com/"));
        assert!(is_root_url("http://example.com"));
        assert!(is_root_url("http://example.com/"));

        assert!(!is_root_url("https://example.com/path"));
        assert!(!is_root_url("https://example.com/?query=value"));
        assert!(!is_root_url("https://example.com/#fragment"));
        assert!(!is_root_url("https://example.com/path?query=value"));
        assert!(!is_root_url("invalid-url"));
    }
}
