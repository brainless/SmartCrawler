use crate::html_parser::HtmlNode;
use crate::utils::extract_domain_from_url;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchStatus {
    Pending,
    InProgress,
    Success,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlData {
    pub url: String,
    pub domain: String,
    pub status: FetchStatus,
    pub html_source: Option<String>,
    pub html_tree: Option<HtmlNode>,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UrlData {
    pub fn new(url: String) -> Self {
        let domain = extract_domain_from_url(&url).unwrap_or_else(|| "unknown".to_string());
        let now = Utc::now();

        UrlData {
            url,
            domain,
            status: FetchStatus::Pending,
            html_source: None,
            html_tree: None,
            title: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_status(&mut self, status: FetchStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn set_html_data(
        &mut self,
        html_source: String,
        html_tree: HtmlNode,
        title: Option<String>,
    ) {
        self.html_source = Some(html_source);
        self.html_tree = Some(html_tree);
        self.title = title;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Default)]
pub struct UrlStorage {
    urls_by_domain: HashMap<String, HashMap<String, UrlData>>,
}

impl UrlStorage {
    pub fn new() -> Self {
        UrlStorage {
            urls_by_domain: HashMap::new(),
        }
    }

    pub fn add_url(&mut self, url: String) -> bool {
        let domain = extract_domain_from_url(&url).unwrap_or_else(|| "unknown".to_string());

        let domain_urls = self
            .urls_by_domain
            .entry(domain.clone())
            .or_default();

        if domain_urls.contains_key(&url) {
            false // URL already exists
        } else {
            domain_urls.insert(url.clone(), UrlData::new(url));
            true // URL added
        }
    }

    pub fn get_url_data(&self, url: &str) -> Option<&UrlData> {
        let domain = extract_domain_from_url(url)?;
        self.urls_by_domain.get(&domain)?.get(url)
    }

    pub fn get_url_data_mut(&mut self, url: &str) -> Option<&mut UrlData> {
        let domain = extract_domain_from_url(url)?;
        self.urls_by_domain.get_mut(&domain)?.get_mut(url)
    }

    pub fn get_urls_by_domain(&self, domain: &str) -> Option<&HashMap<String, UrlData>> {
        self.urls_by_domain.get(domain)
    }

    pub fn get_all_urls(&self) -> Vec<&UrlData> {
        self.urls_by_domain
            .values()
            .flat_map(|domain_urls| domain_urls.values())
            .collect()
    }

    pub fn get_completed_urls(&self) -> Vec<&UrlData> {
        self.get_all_urls()
            .into_iter()
            .filter(|url_data| matches!(url_data.status, FetchStatus::Success))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_storage_add_url() {
        let mut storage = UrlStorage::new();

        assert!(storage.add_url("https://example.com".to_string()));
        assert!(!storage.add_url("https://example.com".to_string())); // duplicate
        assert!(storage.add_url("https://example.org".to_string()));
    }

    #[test]
    fn test_url_storage_get_url_data() {
        let mut storage = UrlStorage::new();
        storage.add_url("https://example.com".to_string());

        let url_data = storage.get_url_data("https://example.com");
        assert!(url_data.is_some());
        assert_eq!(url_data.unwrap().url, "https://example.com");
        assert_eq!(url_data.unwrap().domain, "example.com");
    }

    #[test]
    fn test_url_storage_get_urls_by_domain() {
        let mut storage = UrlStorage::new();
        storage.add_url("https://example.com/page1".to_string());
        storage.add_url("https://example.com/page2".to_string());
        storage.add_url("https://example.org/page1".to_string());

        let example_com_urls = storage.get_urls_by_domain("example.com");
        assert!(example_com_urls.is_some());
        assert_eq!(example_com_urls.unwrap().len(), 2);

        let example_org_urls = storage.get_urls_by_domain("example.org");
        assert!(example_org_urls.is_some());
        assert_eq!(example_org_urls.unwrap().len(), 1);
    }

    #[test]
    fn test_url_data_update_status() {
        let mut url_data = UrlData::new("https://example.com".to_string());
        let original_time = url_data.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(1));
        url_data.update_status(FetchStatus::InProgress);

        assert!(matches!(url_data.status, FetchStatus::InProgress));
        assert!(url_data.updated_at > original_time);
    }
}
