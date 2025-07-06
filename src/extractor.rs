use crate::content::ExtractionNode;
use scraper::{ElementRef, Html, Node};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct GroupedData {
    pub tag: String,
    pub classes: Vec<String>,
    pub depth: usize,
    pub parent_path: String,
    pub full_path: String,
    pub items: Vec<ExtractionNode>,
}

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

    pub fn find_grouped_data(&self, node: &ExtractionNode) -> Vec<GroupedData> {
        let mut grouped_data = Vec::new();
        self.find_grouped_data_recursive(node, 0, "", &mut grouped_data);

        // Filter out groups with less than 2 items
        grouped_data.retain(|group| group.items.len() >= 2);

        // Deduplicate groups based on their signature
        grouped_data = self.deduplicate_grouped_data(grouped_data);

        // Sort by depth (deeper groups first) and then by number of items (more items first)
        grouped_data.sort_by(|a, b| {
            b.depth
                .cmp(&a.depth)
                .then_with(|| b.items.len().cmp(&a.items.len()))
        });

        grouped_data
    }

    fn find_grouped_data_recursive(
        &self,
        node: &ExtractionNode,
        depth: usize,
        parent_path: &str,
        grouped_data: &mut Vec<GroupedData>,
    ) {
        // Create current simple path and full CSS selector path
        let current_path = if parent_path.is_empty() {
            node.tag.clone()
        } else {
            format!("{}/{}", parent_path, node.tag)
        };

        let current_full_path = if parent_path.is_empty() {
            self.create_element_selector(node)
        } else {
            format!("{} > {}", parent_path, self.create_element_selector(node))
        };

        // Group siblings by tag and class combination
        let mut sibling_groups: HashMap<String, Vec<ExtractionNode>> = HashMap::new();

        for child in &node.children {
            // Create a key based on tag and classes
            let key = format!("{}:{}", child.tag, child.classes.join(","));
            sibling_groups.entry(key).or_default().push(child.clone());
        }

        // Find groups with multiple items
        for (key, items) in sibling_groups {
            if items.len() >= 2 {
                let parts: Vec<&str> = key.split(':').collect();
                let tag = parts[0].to_string();
                let classes = if parts.len() > 1 && !parts[1].is_empty() {
                    parts[1].split(',').map(|s| s.to_string()).collect()
                } else {
                    Vec::new()
                };

                // Check if items have similar structure (similar text content or similar children)
                if self.are_items_similar(&items) {
                    // Create the full path including the grouped element itself
                    // Use the first item to get the complete selector (tag + id + classes)
                    let grouped_element_selector = if let Some(first_item) = items.first() {
                        self.create_element_selector(first_item)
                    } else {
                        tag.clone()
                    };

                    let full_element_path = if current_full_path.is_empty() {
                        grouped_element_selector
                    } else {
                        format!("{current_full_path} > {grouped_element_selector}")
                    };

                    grouped_data.push(GroupedData {
                        tag,
                        classes,
                        depth,
                        parent_path: current_path.clone(),
                        full_path: full_element_path,
                        items,
                    });
                }
            }
        }

        // Recursively process children
        for child in &node.children {
            self.find_grouped_data_recursive(child, depth + 1, &current_full_path, grouped_data);
        }
    }

    fn are_items_similar(&self, items: &[ExtractionNode]) -> bool {
        if items.len() < 2 {
            return false;
        }

        // Check if items have similar structure
        let first_item = &items[0];

        // Simple similarity check: same number of children or similar text patterns
        for item in items.iter().skip(1) {
            // Check if they have similar structure (similar number of children)
            let children_diff =
                (item.children.len() as i32 - first_item.children.len() as i32).abs();

            // Allow some variance in children count
            if children_diff > 2 {
                continue;
            }

            // If both have text, check if they're not identical (to avoid header repetition)
            if let (Some(first_text), Some(item_text)) = (&first_item.text, &item.text) {
                if first_text == item_text {
                    return false; // Identical text suggests it's not grouped data
                }
            }

            // If we reach here, items seem similar enough
            return true;
        }

        true
    }

    fn create_element_selector(&self, node: &ExtractionNode) -> String {
        let mut selector = node.tag.clone();

        // Add ID if present
        if let Some(id) = &node.id {
            selector.push_str(&format!("#{id}"));
        }

        // Add classes if present
        if !node.classes.is_empty() {
            for class in &node.classes {
                selector.push_str(&format!(".{class}"));
            }
        }

        selector
    }

    fn deduplicate_grouped_data(&self, grouped_data: Vec<GroupedData>) -> Vec<GroupedData> {
        let mut seen_signatures = HashSet::new();
        let mut deduplicated = Vec::new();

        for group in grouped_data {
            let signature = self.create_group_signature(&group);
            if seen_signatures.insert(signature) {
                deduplicated.push(group);
            }
        }

        deduplicated
    }

    fn create_group_signature(&self, group: &GroupedData) -> String {
        // Create a unique signature based on:
        // 1. Tag and classes
        // 2. Depth and parent path
        // 3. Number of items
        // 4. Sample of text content from items (to distinguish structurally similar but content-different groups)

        let mut signature = format!(
            "{}:{}:{}:{}:{}",
            group.tag,
            group.classes.join(","),
            group.depth,
            group.parent_path,
            group.items.len()
        );

        // Add a sample of text content from the first few items to distinguish groups
        for (i, item) in group.items.iter().take(3).enumerate() {
            if let Some(text) = &item.text {
                // Use first 20 characters of text as part of signature
                let text_sample = if text.len() > 20 { &text[..20] } else { text };
                signature.push_str(&format!(":item{i}:{text_sample}"));
            } else if let Some(first_child_text) = Self::get_first_text_content(item) {
                let text_sample = if first_child_text.len() > 20 {
                    &first_child_text[..20]
                } else {
                    &first_child_text
                };
                signature.push_str(&format!(":item{i}:{text_sample}"));
            }
        }

        signature
    }

    pub fn print_tree(&self, node: &ExtractionNode) {
        Self::print_tree_recursive(node, 0, true);
    }

    pub fn print_grouped_data(&self, grouped_data: &[GroupedData]) {
        if grouped_data.is_empty() {
            println!("No grouped data found.");
            return;
        }

        println!("Found {} grouped data patterns:", grouped_data.len());
        println!();

        for (i, group) in grouped_data.iter().enumerate() {
            println!("Group {} ({} items):", i + 1, group.items.len());
            println!("  Path: {}", group.full_path);
            println!("  Items:");

            for (j, item) in group.items.iter().enumerate() {
                print!("    [{}] ", j + 1);
                if let Some(text) = &item.text {
                    let truncated = if text.len() > 80 {
                        format!("{}...", &text[..77])
                    } else {
                        text.clone()
                    };
                    println!("{truncated}");
                } else if !item.children.is_empty() {
                    // Show first child's text if available
                    if let Some(child_text) = Self::get_first_text_content(item) {
                        let truncated = if child_text.len() > 80 {
                            format!("{}...", &child_text[..77])
                        } else {
                            child_text
                        };
                        println!("{truncated}");
                    } else {
                        println!("<{} with {} children>", item.tag, item.children.len());
                    }
                } else {
                    println!("<empty {}>", item.tag);
                }
            }
            println!();
        }
    }

    fn get_first_text_content(node: &ExtractionNode) -> Option<String> {
        if let Some(text) = &node.text {
            return Some(text.clone());
        }

        for child in &node.children {
            if let Some(text) = Self::get_first_text_content(child) {
                return Some(text);
            }
        }

        None
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
