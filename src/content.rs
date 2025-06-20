use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedContent {
    pub url: String,
    pub title: Option<String>,
    pub text_content: String,
    pub links: Vec<String>,
    pub meta_description: Option<String>,
    pub headings: Vec<String>,
}
