use crate::content::ExtractionNode;
use scraper::{ElementRef, Html, Node};
use std::collections::HashSet;

pub struct HtmlExtractor {
    ignored_tags: HashSet<String>,
    ignored_classes: HashSet<String>,
}

impl HtmlExtractor {
    pub fn new() -> Self {
        let mut ignored_tags = HashSet::new();
        ignored_tags.insert("script".to_string());
        ignored_tags.insert("style".to_string());
        ignored_tags.insert("meta".to_string());
        ignored_tags.insert("link".to_string());
        ignored_tags.insert("noscript".to_string());
        ignored_tags.insert("title".to_string());
        ignored_tags.insert("head".to_string());
        ignored_tags.insert("img".to_string());
        ignored_tags.insert("video".to_string());
        ignored_tags.insert("audio".to_string());
        ignored_tags.insert("svg".to_string());
        ignored_tags.insert("path".to_string());
        ignored_tags.insert("iframe".to_string());
        ignored_tags.insert("embed".to_string());
        ignored_tags.insert("object".to_string());

        let mut ignored_classes = HashSet::new();
        ignored_classes.insert("active".to_string());
        ignored_classes.insert("highlighted".to_string());
        ignored_classes.insert("selected".to_string());

        Self {
            ignored_tags,
            ignored_classes,
        }
    }

    pub fn extract(&self, html: &str) -> Option<ExtractionNode> {
        let document = Html::parse_document(html);

        // Find the root element (usually html or body)
        let root = document.root_element();
        self.process_element(&root)
    }

    fn process_element(&self, element: &ElementRef) -> Option<ExtractionNode> {
        let tag_name = element.value().name().to_lowercase();

        // Apply tag filtering rules
        if self.should_ignore_tag(&tag_name) {
            return None;
        }

        // Get element attributes
        let id = self.process_id_attribute(element);
        let classes = self.process_class_attribute(element);

        // Get direct text content (not from children)
        let direct_text = self.extract_direct_text(element);

        // Process children
        let mut children = Vec::new();
        for child in element.children() {
            if let Some(child_element) = ElementRef::wrap(child) {
                if let Some(child_node) = self.process_element(&child_element) {
                    children.push(child_node);
                }
            }
        }

        // Apply text rules - merge sibling paragraphs if needed
        let children = self.merge_sibling_paragraphs(children);

        // Check if element should be ignored due to emptiness
        if self.should_ignore_empty_element(&direct_text, &children) {
            return None;
        }

        Some(ExtractionNode {
            tag: tag_name,
            id,
            classes,
            text: direct_text,
            children,
        })
    }

    fn should_ignore_tag(&self, tag: &str) -> bool {
        self.ignored_tags.contains(tag)
    }

    fn should_ignore_empty_element(
        &self,
        text: &Option<String>,
        children: &[ExtractionNode],
    ) -> bool {
        // Ignore if no text and no children
        if text.is_none() && children.is_empty() {
            return true;
        }

        // Ignore if text only contains control/special characters
        if let Some(text_content) = text {
            if text_content
                .chars()
                .all(|c| c.is_control() || c.is_whitespace())
            {
                return children.is_empty();
            }
        }

        false
    }

    fn process_id_attribute(&self, element: &ElementRef) -> Option<String> {
        element
            .value()
            .attr("id")
            .map(|id| id.trim().to_string())
            .filter(|id| !id.is_empty())
    }

    fn process_class_attribute(&self, element: &ElementRef) -> Vec<String> {
        element
            .value()
            .attr("class")
            .map(|classes| {
                classes
                    .split_whitespace()
                    .map(|class| class.trim().to_string())
                    .filter(|class| !class.is_empty() && !self.ignored_classes.contains(class))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn extract_direct_text(&self, element: &ElementRef) -> Option<String> {
        let mut direct_text = String::new();

        for child in element.children() {
            if let Node::Text(text_node) = child.value() {
                direct_text.push_str(text_node.text.as_ref());
            }
        }

        let trimmed = direct_text.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }

    fn merge_sibling_paragraphs(&self, children: Vec<ExtractionNode>) -> Vec<ExtractionNode> {
        if children.len() < 2 {
            return children;
        }

        let mut merged = Vec::new();
        let mut i = 0;

        while i < children.len() {
            let mut current = children[i].clone();

            // Check if current element is a paragraph and has text
            if current.tag == "p" && current.text.is_some() {
                let mut merged_text = current.text.clone().unwrap_or_default();
                let mut j = i + 1;

                // Look for consecutive paragraph siblings
                while j < children.len() && children[j].tag == "p" && children[j].text.is_some() {
                    if !merged_text.is_empty() {
                        merged_text.push(' ');
                    }
                    merged_text.push_str(children[j].text.as_ref().unwrap());
                    j += 1;
                }

                // If we merged paragraphs, update the current node
                if j > i + 1 {
                    current.text = Some(merged_text);
                    i = j;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }

            merged.push(current);
        }

        merged
    }

    pub fn print_tree(&self, node: &ExtractionNode) {
        Self::print_tree_recursive(node, 0, true);
    }

    fn print_tree_recursive(node: &ExtractionNode, depth: usize, is_last: bool) {
        // Create tree line prefix
        let mut prefix = String::new();
        for i in 0..depth {
            if i == depth - 1 {
                if is_last {
                    prefix.push_str("└── ");
                } else {
                    prefix.push_str("├── ");
                }
            } else {
                prefix.push_str("│   ");
            }
        }

        // Build element display string
        let mut display = format!("{}{}", prefix, node.tag);

        if let Some(id) = &node.id {
            display.push_str(&format!(" id=\"{id}\""));
        }

        if !node.classes.is_empty() {
            display.push_str(&format!(" class=\"{}\"", node.classes.join(" ")));
        }

        if let Some(text) = &node.text {
            // Truncate long text for display
            let truncated = if text.len() > 50 {
                format!("{}...", &text[..47])
            } else {
                text.clone()
            };
            display.push_str(&format!(" [{truncated}]"));
        }

        println!("{display}");

        // Print children
        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            Self::print_tree_recursive(child, depth + 1, is_last_child);
        }
    }
}

impl Default for HtmlExtractor {
    fn default() -> Self {
        Self::new()
    }
}
