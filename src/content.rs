use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionNode {
    pub tag: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub text: Option<String>,
    pub children: Vec<ExtractionNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedWebPage {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub links: Vec<String>,
    pub meta_description: Option<String>,
    pub headings: Vec<String>,
    pub page_node_tree: Option<ExtractionNode>,
    pub filtered_node_tree: Option<ExtractionNode>,
}

/// Container for storing node trees per domain to identify duplicates
#[derive(Debug, Clone)]
pub struct DomainNodeContainer {
    pub domain: String,
    pub page_trees: HashMap<String, ExtractionNode>, // URL -> node tree
    pub duplicate_node_signatures: HashSet<String>,
}

impl DomainNodeContainer {
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            page_trees: HashMap::new(),
            duplicate_node_signatures: HashSet::new(),
        }
    }

    /// Add a page's node tree and update duplicate signatures
    pub fn add_page_tree(&mut self, url: String, tree: ExtractionNode) {
        // If this is not the first page, find duplicates
        if !self.page_trees.is_empty() {
            self.find_and_mark_duplicates(&tree);
        }

        self.page_trees.insert(url, tree);
    }

    /// Find duplicate nodes between the new tree and existing trees
    fn find_and_mark_duplicates(&mut self, new_tree: &ExtractionNode) {
        let existing_trees: Vec<_> = self.page_trees.values().cloned().collect();
        for existing_tree in existing_trees {
            self.find_duplicates_recursive(&existing_tree, new_tree);
        }
    }

    /// Recursively find duplicate nodes between two trees
    fn find_duplicates_recursive(&mut self, tree1: &ExtractionNode, tree2: &ExtractionNode) {
        // Create signatures for all nodes in both trees and find matches
        let mut tree1_signatures = HashSet::new();
        let mut tree2_signatures = HashSet::new();

        self.collect_node_signatures(tree1, &mut tree1_signatures);
        self.collect_node_signatures(tree2, &mut tree2_signatures);

        // Find intersection (duplicates)
        for signature in tree1_signatures.intersection(&tree2_signatures) {
            self.duplicate_node_signatures.insert(signature.clone());
        }
    }

    /// Collect all node signatures from a tree
    fn collect_node_signatures(&self, node: &ExtractionNode, signatures: &mut HashSet<String>) {
        let signature = self.create_node_signature(node);
        signatures.insert(signature);

        for child in &node.children {
            self.collect_node_signatures(child, signatures);
        }
    }

    /// Create a unique signature for a node based on tag, classes, and structure
    fn create_node_signature(&self, node: &ExtractionNode) -> String {
        format!(
            "{}:{}:{}:{}",
            node.tag,
            node.classes.join(","),
            node.id.as_deref().unwrap_or(""),
            node.children.len()
        )
    }

    /// Filter out duplicate nodes from a tree, returning only unique content
    pub fn filter_duplicates(&self, tree: &ExtractionNode) -> Option<ExtractionNode> {
        self.filter_node_recursive(tree)
    }

    /// Recursively filter out duplicate nodes
    fn filter_node_recursive(&self, node: &ExtractionNode) -> Option<ExtractionNode> {
        let signature = self.create_node_signature(node);

        // If this node is a duplicate, skip it
        if self.duplicate_node_signatures.contains(&signature) {
            return None;
        }

        // Filter children
        let filtered_children: Vec<ExtractionNode> = node
            .children
            .iter()
            .filter_map(|child| self.filter_node_recursive(child))
            .collect();

        Some(ExtractionNode {
            tag: node.tag.clone(),
            id: node.id.clone(),
            classes: node.classes.clone(),
            text: node.text.clone(),
            children: filtered_children,
        })
    }
}
