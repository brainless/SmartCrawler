use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClaudeError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Environment variable ANTHROPIC_API_KEY not found")]
    MissingApiKey,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeResponse {
    pub id: String,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug)]
pub struct ClaudeClient {
    client: Client,
    api_key: String,
}

impl ClaudeClient {
    pub fn new() -> Result<Self, ClaudeError> {
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| ClaudeError::MissingApiKey)?;

        let client = Client::builder()
            .user_agent("Smart-Crawler/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Ok(Self { client, api_key })
    }

    pub async fn select_urls<T>(
        &self,
        objective: &str,
        urls: &[T],
        domain: &str,
        max_urls: usize,
    ) -> Result<Vec<String>, ClaudeError>
    where
        T: AsRef<str>,
    {
        let url_list: Vec<String> = urls.iter()
            .take(200) // Limit to avoid token limits
            .map(|u| u.as_ref().to_string())
            .collect();

        let prompt = format!(
            r#"You are helping a web crawler select the most relevant URLs to crawl for a specific objective.

Domain: {}
Objective: {}

Here are the available URLs from the sitemap:
{}

Please analyze these URLs and select the {} most relevant ones that would likely contain information related to the objective. 

Consider:
1. URL structure and path names that suggest relevant content
2. Likely page types (product pages, articles, documentation, etc.)
3. Depth and specificity of URLs
4. Avoid redundant or overly similar URLs

Return ONLY a JSON array of the selected URLs, nothing else. Example format:
["https://example.com/page1", "https://example.com/page2"]"#,
            domain,
            objective,
            url_list.join("\n"),
            max_urls.min(20) // Conservative limit
        );

        let response = self.send_message(&prompt).await?;
        
        // Parse the JSON response
        let content = response.content.first()
            .ok_or_else(|| ClaudeError::ApiError("No content in response".to_string()))?;

        let selected_urls: Vec<String> = serde_json::from_str(&content.text)
            .map_err(|_| ClaudeError::ApiError("Failed to parse URL selection response".to_string()))?;

        Ok(selected_urls)
    }

    pub async fn analyze_content(
        &self,
        objective: &str,
        url: &str,
        content: &str,
    ) -> Result<String, ClaudeError> {
        let prompt = format!(
            r#"You are analyzing web content for a specific objective.

URL: {}
Objective: {}

Content (truncated if necessary):
{}

Please analyze this content and extract information relevant to the objective. Provide a clear, structured response with:
1. Whether this page contains relevant information for the objective
2. Key findings or extracted data
3. Any actionable insights

Keep the response concise but informative."#,
            url,
            objective,
            content.chars().take(8000).collect::<String>() // Limit content length
        );

        let response = self.send_message(&prompt).await?;
        
        let content = response.content.first()
            .ok_or_else(|| ClaudeError::ApiError("No content in response".to_string()))?;

        Ok(content.text.clone())
    }

    pub async fn send_message(&self, message: &str) -> Result<ClaudeResponse, ClaudeError> {
        let payload = json!({
            "model": "claude-3-haiku-20240307",
            "max_tokens": 2000,
            "messages": [
                {
                    "role": "user",
                    "content": message
                }
            ]
        });

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ClaudeError::ApiError(format!("API request failed: {}", error_text)));
        }

        let claude_response: ClaudeResponse = response.json().await?;
        Ok(claude_response)
    }
}