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

pub fn is_numeric_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_digit())
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
    fn test_is_numeric_id() {
        assert!(is_numeric_id("123"));
        assert!(is_numeric_id("0"));
        assert!(!is_numeric_id("abc"));
        assert!(!is_numeric_id("12a"));
        assert!(!is_numeric_id(""));
    }
}
