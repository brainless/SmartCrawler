use crate::html_parser::HtmlNode;
use crate::utils::extract_domain_from_url;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

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
    domain_duplicates: HashMap<String, DomainDuplicates>,
}

impl UrlStorage {
    pub fn new() -> Self {
        UrlStorage {
            urls_by_domain: HashMap::new(),
            domain_duplicates: HashMap::new(),
        }
    }

    pub fn add_url(&mut self, url: String) -> bool {
        let domain = extract_domain_from_url(&url).unwrap_or_else(|| "unknown".to_string());

        let domain_urls = self.urls_by_domain.entry(domain.clone()).or_default();

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

    pub fn analyze_domain_duplicates(&mut self, domain: &str) {
        if let Some(domain_urls) = self.urls_by_domain.get(domain) {
            let completed_urls: Vec<_> = domain_urls
                .values()
                .filter(|url_data| matches!(url_data.status, FetchStatus::Success))
                .collect();

            if completed_urls.len() < 2 {
                return; // Need at least 2 pages to find duplicates
            }

            let mut node_occurrence_count: HashMap<NodeSignature, usize> = HashMap::new();

            // Count occurrences of each node signature across all pages
            for url_data in &completed_urls {
                if let Some(html_tree) = &url_data.html_tree {
                    Self::collect_node_signatures(html_tree, &mut node_occurrence_count);
                }
            }

            // Mark nodes that appear in 2 or more pages as duplicates
            let domain_duplicates = self
                .domain_duplicates
                .entry(domain.to_string())
                .or_default();
            for (signature, count) in node_occurrence_count {
                if count >= 2 {
                    domain_duplicates.add_duplicate_node(signature);
                }
            }
        }
    }

    fn collect_node_signatures(node: &HtmlNode, signatures: &mut HashMap<NodeSignature, usize>) {
        // Skip structural/container elements that naturally appear on every page
        if !Self::is_structural_element(&node.tag) {
            let signature = NodeSignature::from_html_node(node);
            // Only count nodes with meaningful content or specific styling
            if Self::is_meaningful_node(node) {
                *signatures.entry(signature).or_insert(0) += 1;
            }
        }

        for child in &node.children {
            Self::collect_node_signatures(child, signatures);
        }
    }

    fn is_structural_element(tag: &str) -> bool {
        matches!(
            tag,
            "html" | "head" | "body" | "main" | "article" | "section"
        )
    }

    fn is_meaningful_node(node: &HtmlNode) -> bool {
        // Consider a node meaningful if it has:
        // - Non-empty content (text content or children), OR
        // - Specific CSS classes/IDs that indicate styling, OR
        // - Is a semantic element that likely appears across multiple pages
        (!node.content.trim().is_empty() || !node.children.is_empty())
            || !node.classes.is_empty()
            || node.id.is_some()
            || matches!(
                node.tag.as_str(),
                "nav"
                    | "header"
                    | "footer"
                    | "aside"
                    | "form"
                    | "button"
                    | "a"
                    | "ul"
                    | "ol"
                    | "menu"
            )
    }

    pub fn get_domain_duplicates(&self, domain: &str) -> Option<&DomainDuplicates> {
        self.domain_duplicates.get(domain)
    }

    pub fn add_urls_from_same_domain(&mut self, urls: Vec<String>) {
        for url in urls {
            self.add_url(url);
        }
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

    #[test]
    fn test_add_urls_from_same_domain() {
        let mut storage = UrlStorage::new();
        let urls = vec![
            "https://example.com/page1".to_string(),
            "https://example.com/page2".to_string(),
            "https://example.com/page3".to_string(),
        ];

        storage.add_urls_from_same_domain(urls);

        let example_com_urls = storage.get_urls_by_domain("example.com");
        assert!(example_com_urls.is_some());
        assert_eq!(example_com_urls.unwrap().len(), 3);
    }

    #[test]
    fn test_analyze_domain_duplicates() {
        use crate::html_parser::HtmlParser;

        let mut storage = UrlStorage::new();
        let parser = HtmlParser::new();

        storage.add_url("https://example.com/page1".to_string());
        storage.add_url("https://example.com/page2".to_string());

        // Create mock HTML trees with common elements
        let html1 = r#"<html><body><nav class="navbar">Navigation</nav><div class="content">Page 1 content</div></body></html>"#;
        let html2 = r#"<html><body><nav class="navbar">Navigation</nav><div class="content">Page 2 content</div></body></html>"#;

        let tree1 = parser.parse(html1);
        let tree2 = parser.parse(html2);

        // Set the HTML data for both URLs
        if let Some(url_data) = storage.get_url_data_mut("https://example.com/page1") {
            url_data.set_html_data(html1.to_string(), tree1, Some("Page 1".to_string()));
            url_data.update_status(FetchStatus::Success);
        }

        if let Some(url_data) = storage.get_url_data_mut("https://example.com/page2") {
            url_data.set_html_data(html2.to_string(), tree2, Some("Page 2".to_string()));
            url_data.update_status(FetchStatus::Success);
        }

        // Analyze domain duplicates
        storage.analyze_domain_duplicates("example.com");

        let duplicates = storage.get_domain_duplicates("example.com");
        assert!(duplicates.is_some());
        assert!(duplicates.unwrap().get_duplicate_count() > 0);
    }

    #[test]
    fn test_node_signature_creation() {
        use crate::html_parser::HtmlNode;

        let node = HtmlNode::new(
            "div".to_string(),
            vec!["container".to_string(), "main".to_string()],
            Some("content".to_string()),
            "Test content".to_string(),
        );

        let signature = NodeSignature::from_html_node(&node);
        assert_eq!(signature.tag, "div");
        assert_eq!(signature.classes, vec!["container", "main"]);
        assert_eq!(signature.id, Some("content".to_string()));
        assert_eq!(signature.content, "Test content");
        assert!(!signature.content_hash.is_empty());
    }

    #[test]
    fn test_domain_duplicates_detection() {
        let mut duplicates = DomainDuplicates::new();

        let signature = NodeSignature {
            tag: "nav".to_string(),
            classes: vec!["navbar".to_string()],
            id: None,
            content: "Navigation".to_string(),
            content_hash: "test_hash".to_string(),
        };

        assert!(!duplicates.is_duplicate(&signature));

        duplicates.add_duplicate_node(signature.clone());
        assert!(duplicates.is_duplicate(&signature));
        assert_eq!(duplicates.get_duplicate_count(), 1);
    }

    #[test]
    fn test_content_hash_includes_children() {
        use crate::html_parser::HtmlParser;

        let parser = HtmlParser::new();

        // Two divs with same tag/class but different children
        let html1 = r#"<div class="container"><p>Content 1</p></div>"#;
        let html2 = r#"<div class="container"><p>Content 2</p></div>"#;
        let html3 = r#"<div class="container"><p>Content 1</p></div>"#; // Same as html1

        let node1 = parser.parse(html1);
        let node2 = parser.parse(html2);
        let node3 = parser.parse(html3);

        let sig1 = NodeSignature::from_html_node(&node1);
        let sig2 = NodeSignature::from_html_node(&node2);
        let sig3 = NodeSignature::from_html_node(&node3);

        // sig1 and sig2 should be different due to different child content
        assert_ne!(sig1.content_hash, sig2.content_hash);

        // sig1 and sig3 should be identical
        assert_eq!(sig1.content_hash, sig3.content_hash);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NodeSignature {
    pub tag: String,
    pub classes: Vec<String>,
    pub id: Option<String>,
    pub content: String,
    pub content_hash: String, // Hash of complete structure including children
}

impl NodeSignature {
    pub fn from_html_node(node: &HtmlNode) -> Self {
        let content_hash = Self::compute_content_hash(node);

        NodeSignature {
            tag: node.tag.clone(),
            classes: node.classes.clone(),
            id: node.id.clone(),
            content: node.content.clone(),
            content_hash,
        }
    }

    fn compute_content_hash(node: &HtmlNode) -> String {
        let mut hasher = DefaultHasher::new();

        // Hash the complete structure: tag, classes, id, content, and children structure
        node.tag.hash(&mut hasher);
        node.classes.hash(&mut hasher);
        node.id.hash(&mut hasher);
        node.content.hash(&mut hasher);

        // Recursively hash children structure
        Self::hash_children(&node.children, &mut hasher);

        format!("{:x}", hasher.finish())
    }

    fn hash_children(children: &[HtmlNode], hasher: &mut DefaultHasher) {
        for child in children {
            child.tag.hash(hasher);
            child.classes.hash(hasher);
            child.id.hash(hasher);
            child.content.hash(hasher);
            Self::hash_children(&child.children, hasher);
        }
    }
}

#[derive(Debug, Default)]
pub struct DomainDuplicates {
    duplicate_nodes: HashSet<NodeSignature>,
}

impl DomainDuplicates {
    pub fn new() -> Self {
        DomainDuplicates {
            duplicate_nodes: HashSet::new(),
        }
    }

    pub fn add_duplicate_node(&mut self, signature: NodeSignature) {
        self.duplicate_nodes.insert(signature);
    }

    pub fn is_duplicate(&self, signature: &NodeSignature) -> bool {
        self.duplicate_nodes.contains(signature)
    }

    pub fn get_duplicate_count(&self) -> usize {
        self.duplicate_nodes.len()
    }
}
