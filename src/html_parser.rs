use crate::storage::{DomainDuplicates, NodeSignature};
use crate::utils::trim_and_clean_text;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlNode {
    pub tag: String,
    pub classes: Vec<String>,
    pub id: Option<String>,
    pub content: String,
    pub children: Vec<HtmlNode>,
}

impl HtmlNode {
    pub fn new(tag: String, classes: Vec<String>, id: Option<String>, content: String) -> Self {
        HtmlNode {
            tag,
            classes,
            id,
            content,
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: HtmlNode) {
        self.children.push(child);
    }

    pub fn find_title(&self) -> Option<String> {
        if self.tag == "title" && !self.content.is_empty() {
            return Some(self.content.clone());
        }

        for child in &self.children {
            if let Some(title) = child.find_title() {
                return Some(title);
            }
        }

        None
    }

    /// Find elements by CSS-like path (ignoring IDs)
    /// Example: "html body center table tbody tr td table tbody tr.athing.submission td.title"
    pub fn find_by_path(&self, path: &str) -> Vec<&HtmlNode> {
        let path_parts: Vec<&str> = path.split_whitespace().collect();
        if path_parts.is_empty() {
            return vec![];
        }

        let mut results = Vec::new();
        self.find_by_path_recursive(&path_parts, 0, &mut results);
        results
    }

    fn find_by_path_recursive<'a>(
        &'a self,
        path_parts: &[&str],
        depth: usize,
        results: &mut Vec<&'a HtmlNode>,
    ) {
        if depth >= path_parts.len() {
            return;
        }

        let current_part = path_parts[depth];

        // Check if current node matches the current path part
        if self.matches_path_part(current_part) {
            if depth == path_parts.len() - 1 {
                // This is the final part, add to results
                results.push(self);
            } else {
                // Continue searching in children for the next part
                for child in &self.children {
                    child.find_by_path_recursive(path_parts, depth + 1, results);
                }
            }
        }

        // Also check children for the current part (to handle non-matching intermediate nodes)
        for child in &self.children {
            child.find_by_path_recursive(path_parts, depth, results);
        }
    }

    fn matches_path_part(&self, part: &str) -> bool {
        // Parse part like "tr.athing.submission" or just "td"
        if let Some(dot_pos) = part.find('.') {
            let tag = &part[..dot_pos];
            let classes_str = &part[dot_pos + 1..];
            let required_classes: Vec<&str> = classes_str.split('.').collect();

            // Check tag matches and all required classes are present
            self.tag == tag
                && required_classes
                    .iter()
                    .all(|class| self.classes.contains(&class.to_string()))
        } else {
            // Just a tag name
            self.tag == part
        }
    }
}

pub struct HtmlParser {
    ignored_tags: HashSet<String>,
}

impl HtmlParser {
    pub fn new() -> Self {
        let mut ignored_tags = HashSet::new();
        ignored_tags.extend(
            [
                "script", "style", "noscript", "svg", "path", "img", "video", "audio", "canvas",
                "embed", "object", "iframe",
            ]
            .iter()
            .map(|s| s.to_string()),
        );

        HtmlParser { ignored_tags }
    }

    pub fn parse(&self, html: &str) -> HtmlNode {
        let document = Html::parse_document(html);
        let html_selector = Selector::parse("html").unwrap();

        if let Some(html_element) = document.select(&html_selector).next() {
            self.parse_element(html_element)
        } else {
            let body_selector = Selector::parse("body").unwrap();
            if let Some(body_element) = document.select(&body_selector).next() {
                self.parse_element(body_element)
            } else {
                HtmlNode::new("html".to_string(), vec![], None, String::new())
            }
        }
    }

    fn parse_element(&self, element: ElementRef) -> HtmlNode {
        let tag = element.value().name().to_string();

        if self.ignored_tags.contains(&tag) {
            return HtmlNode::new(tag, vec![], None, String::new());
        }

        let classes = self.extract_classes(element);
        let id = self.extract_id(element);

        let mut children = Vec::new();

        for child in element.children() {
            if let Some(child_element) = ElementRef::wrap(child) {
                let child_node = self.parse_element(child_element);

                if !self.is_blank_node(&child_node) {
                    children.push(child_node);
                }
            }
        }

        let content = if children.is_empty() {
            trim_and_clean_text(&self.extract_text_content(element))
        } else {
            String::new()
        };

        let mut node = HtmlNode::new(tag, classes, id, content);
        node.children = children;
        node
    }

    fn extract_classes(&self, element: ElementRef) -> Vec<String> {
        element
            .value()
            .attr("class")
            .unwrap_or("")
            .split_whitespace()
            .map(|class| class.trim().to_string())
            .filter(|class| !class.is_empty())
            .collect()
    }

    fn extract_id(&self, element: ElementRef) -> Option<String> {
        element
            .value()
            .attr("id")
            .map(|id| id.trim().to_string())
            .filter(|id| !id.is_empty())
    }

    fn extract_text_content(&self, element: ElementRef) -> String {
        element.text().collect::<Vec<_>>().join(" ")
    }

    fn is_blank_node(&self, node: &HtmlNode) -> bool {
        node.content.trim().is_empty() && node.children.is_empty()
    }

    pub fn filter_domain_duplicates(
        node: &HtmlNode,
        domain_duplicates: &DomainDuplicates,
    ) -> HtmlNode {
        let signature = NodeSignature::from_html_node(node);

        // Create the filtered node structure
        let mut filtered_node = HtmlNode::new(
            node.tag.clone(),
            node.classes.clone(),
            node.id.clone(),
            if domain_duplicates.is_duplicate(&signature) {
                "[FILTERED DUPLICATE]".to_string()
            } else {
                node.content.clone()
            },
        );

        // Always process children to maintain structure
        for child in &node.children {
            let filtered_child = Self::filter_domain_duplicates(child, domain_duplicates);
            filtered_node.add_child(filtered_child);
        }

        filtered_node
    }

    pub fn extract_links(&self, html: &str, base_domain: &str) -> Vec<String> {
        let document = Html::parse_document(html);
        let link_selector = Selector::parse("a[href]").unwrap();
        let mut links = HashSet::new();

        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(url) = self.resolve_url(href, base_domain) {
                    if self.is_same_domain(&url, base_domain) {
                        links.insert(url);
                    }
                }
            }
        }

        links.into_iter().collect()
    }

    fn resolve_url(&self, href: &str, base_domain: &str) -> Result<String, String> {
        if href.starts_with("http://") || href.starts_with("https://") {
            Ok(href.to_string())
        } else if href.starts_with('/') {
            Ok(format!("https://{base_domain}{href}"))
        } else if href.starts_with("//") {
            Ok(format!("https:{href}"))
        } else {
            Ok(format!("https://{base_domain}/{href}"))
        }
    }

    fn is_same_domain(&self, url: &str, base_domain: &str) -> bool {
        if let Ok(parsed_url) = Url::parse(url) {
            if let Some(host) = parsed_url.host_str() {
                return host == base_domain || host.ends_with(&format!(".{base_domain}"));
            }
        }
        false
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_parser_basic() {
        let parser = HtmlParser::new();
        let html = r#"<html><body><h1>Title</h1><p>Content</p></body></html>"#;
        let node = parser.parse(html);

        assert_eq!(node.tag, "html");
        assert_eq!(node.children.len(), 1); // body
        let body = &node.children[0];
        assert_eq!(body.tag, "body");
        assert_eq!(body.children.len(), 2);
        assert_eq!(body.children[0].tag, "h1");
        assert_eq!(body.children[0].content, "Title");
        assert_eq!(body.children[1].tag, "p");
        assert_eq!(body.children[1].content, "Content");
    }

    #[test]
    fn test_html_parser_ignores_scripts() {
        let parser = HtmlParser::new();
        let html = r#"<html><body><script>alert('test');</script><p>Content</p></body></html>"#;
        let node = parser.parse(html);

        let body = &node.children[0];
        assert_eq!(body.children.len(), 1);
        assert_eq!(body.children[0].tag, "p");
    }

    #[test]
    fn test_html_parser_classes_and_ids() {
        let parser = HtmlParser::new();
        let html =
            r#"<html><body><div class="container main" id="content">Text</div></body></html>"#;
        let node = parser.parse(html);

        let body = &node.children[0];
        assert_eq!(body.children.len(), 1);
        let div_node = &body.children[0];
        assert_eq!(div_node.tag, "div");
        assert_eq!(div_node.classes, vec!["container", "main"]);
        assert_eq!(div_node.id, Some("content".to_string()));
        assert_eq!(div_node.content, "Text");
    }

    #[test]
    fn test_html_parser_preserves_numeric_ids() {
        let parser = HtmlParser::new();
        let html = r#"<html><body><div id="123">Text</div></body></html>"#;
        let node = parser.parse(html);

        let body = &node.children[0];
        assert_eq!(body.children.len(), 1);
        let div_node = &body.children[0];
        assert_eq!(div_node.id, Some("123".to_string()));
    }

    #[test]
    fn test_html_parser_merges_text_siblings() {
        let parser = HtmlParser::new();
        let html = r#"<html><body><p>First</p><p>Second</p><div>Different</div></body></html>"#;
        let node = parser.parse(html);

        let body = &node.children[0];
        assert_eq!(body.children.len(), 3); // p, p, div
    }

    #[test]
    fn test_find_title() {
        let parser = HtmlParser::new();
        let html = r#"<html><head><title>Page Title</title></head><body>Content</body></html>"#;
        let node = parser.parse(html);

        let title = node.find_title();
        assert_eq!(title, Some("Page Title".to_string()));
    }

    #[test]
    fn test_html_parser_blank_nodes() {
        let parser = HtmlParser::new();
        let html = r#"<html><body><div></div><p>Content</p></body></html>"#;
        let node = parser.parse(html);

        let body = &node.children[0];
        assert_eq!(body.children.len(), 1);
        assert_eq!(body.children[0].tag, "p");
    }

    #[test]
    fn test_extract_links() {
        let parser = HtmlParser::new();
        let html = r#"<html><body>
            <a href="/page1">Link 1</a>
            <a href="https://example.com/page2">Link 2</a>
            <a href="https://other.com/page3">External Link</a>
            <a href="//example.com/page4">Protocol-relative</a>
        </body></html>"#;

        let links = parser.extract_links(html, "example.com");

        assert!(links.contains(&"https://example.com/page1".to_string()));
        assert!(links.contains(&"https://example.com/page2".to_string()));
        // Protocol-relative URLs are handled correctly
        assert!(links.iter().any(|link| link.contains("page4")));
        assert!(!links.iter().any(|link| link.contains("other.com")));
    }

    #[test]
    fn test_filter_domain_duplicates() {
        use crate::storage::{DomainDuplicates, NodeSignature};

        let parser = HtmlParser::new();
        let html = r#"<html><body><nav class="navbar">Navigation</nav><div class="content">Main content</div></body></html>"#;
        let node = parser.parse(html);

        let mut duplicates = DomainDuplicates::new();

        // Find the nav element in the parsed tree and get its signature
        let body = &node.children[0];
        let nav_node = &body.children[0]; // The nav element
        let nav_signature = NodeSignature::from_html_node(nav_node);
        duplicates.add_duplicate_node(nav_signature);

        let filtered = HtmlParser::filter_domain_duplicates(&node, &duplicates);

        // The structure should be preserved, but nav content should be marked as filtered
        assert_eq!(filtered.tag, "html");
        let body = &filtered.children[0];
        assert_eq!(body.tag, "body");
        assert_eq!(body.children.len(), 2); // Both nav and div should remain
        assert_eq!(body.children[0].tag, "nav");
        assert_eq!(body.children[0].content, "[FILTERED DUPLICATE]");
        assert_eq!(body.children[1].tag, "div");
        assert_eq!(body.children[1].content, "Main content");
    }

    #[test]
    fn test_is_same_domain() {
        let parser = HtmlParser::new();

        assert!(parser.is_same_domain("https://example.com/page", "example.com"));
        assert!(parser.is_same_domain("https://sub.example.com/page", "example.com"));
        assert!(!parser.is_same_domain("https://other.com/page", "example.com"));
        assert!(!parser.is_same_domain("https://notexample.com/page", "example.com"));
    }

    #[test]
    fn test_find_by_path() {
        let parser = HtmlParser::new();
        let html = r#"<html>
            <body>
                <center>
                    <table>
                        <tbody>
                            <tr>
                                <td>
                                    <table>
                                        <tbody>
                                            <tr class="athing submission">
                                                <td class="title">First Item</td>
                                            </tr>
                                            <tr class="athing submission">
                                                <td class="title">Second Item</td>
                                            </tr>
                                        </tbody>
                                    </table>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </center>
            </body>
        </html>"#;
        let tree = parser.parse(html);

        // Test finding elements by path
        let results = tree.find_by_path(
            "html body center table tbody tr td table tbody tr.athing.submission td.title",
        );
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].content, "First Item");
        assert_eq!(results[1].content, "Second Item");

        // Test finding by simple tag path
        let body_results = tree.find_by_path("html body");
        assert_eq!(body_results.len(), 1);
        assert_eq!(body_results[0].tag, "body");

        // Test non-existent path
        let empty_results = tree.find_by_path("html body div.nonexistent");
        assert_eq!(empty_results.len(), 0);
    }

    #[test]
    fn test_matches_path_part() {
        let node = HtmlNode::new(
            "tr".to_string(),
            vec!["athing".to_string(), "submission".to_string()],
            None,
            String::new(),
        );

        // Test matching with classes
        assert!(node.matches_path_part("tr.athing.submission"));
        assert!(node.matches_path_part("tr.athing"));
        assert!(node.matches_path_part("tr"));

        // Test non-matching
        assert!(!node.matches_path_part("td.athing"));
        assert!(!node.matches_path_part("tr.nonexistent"));

        // Test simple tag matching
        let simple_node = HtmlNode::new("div".to_string(), vec![], None, String::new());
        assert!(simple_node.matches_path_part("div"));
        assert!(!simple_node.matches_path_part("span"));
    }
}
