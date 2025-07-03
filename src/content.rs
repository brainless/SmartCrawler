use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub extraction_data: Option<ExtractionNode>,
}
