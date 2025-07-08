use crate::utils::{is_numeric_id, trim_and_clean_text};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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

                if !self.is_blank_node(&child_node)
                    && !self.is_duplicate_node(&child_node, &children)
                {
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
            .filter(|id| !id.is_empty() && !is_numeric_id(id))
    }

    fn extract_text_content(&self, element: ElementRef) -> String {
        element.text().collect::<Vec<_>>().join(" ")
    }

    fn is_blank_node(&self, node: &HtmlNode) -> bool {
        node.content.trim().is_empty() && node.children.is_empty()
    }

    fn is_duplicate_node(&self, node: &HtmlNode, existing_children: &[HtmlNode]) -> bool {
        existing_children.iter().any(|existing| {
            existing.tag == node.tag
                && existing.classes == node.classes
                && existing.id == node.id
                && existing.content == node.content
        })
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
    fn test_html_parser_ignores_numeric_ids() {
        let parser = HtmlParser::new();
        let html = r#"<html><body><div id="123">Text</div></body></html>"#;
        let node = parser.parse(html);

        let body = &node.children[0];
        assert_eq!(body.children.len(), 1);
        let div_node = &body.children[0];
        assert_eq!(div_node.id, None);
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
}
