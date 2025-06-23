use crate::llm::{LlmError, LLM}; // Import LLM trait and LlmError
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
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

#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone
pub struct ClaudeResponse {
    pub id: String,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone
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
        let api_key = env::var("ANTHROPIC_API_KEY").map_err(|_| ClaudeError::MissingApiKey)?;

        let client = Client::builder()
            .user_agent("Smart-Crawler/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Ok(Self { client, api_key })
    }

    // This is the original send_message, renamed to avoid conflict with trait method
    // and to keep its specific error type for internal calls.
    async fn send_message_internal(&self, message: &str) -> Result<ClaudeResponse, ClaudeError> {
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
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ClaudeError::ApiError(format!(
                "API request failed: {}",
                error_text
            )));
        }

        let claude_response: ClaudeResponse = response.json().await?;
        Ok(claude_response)
    }
}

#[async_trait]
impl LLM for ClaudeClient {
    // Signature changed to match the updated LLM trait (urls: &[String])
    async fn select_urls(
        &self,
        objective: &str,
        urls: &[String], // Changed from urls: &[T]
        domain: &str,
        max_urls: usize,
    ) -> Result<Vec<String>, LlmError> {
        // url_list is now directly from the `urls` parameter.
        // We clone it to own the data and be able to call .join() later.
        // Also apply the take(200) limit like before.
        let url_list: Vec<String> = urls.iter().take(200).cloned().collect();

        tracing::info!("URLs provided to Claude for selection: {:?}", url_list);

        let prompt = format!(
            r#"You are helping a web crawler select the most relevant URLs to crawl for a specific objective.

Domain: {}
Objective: {}

Here are the available URLs:
{}

Please analyze these URLs and select the {} most relevant ones that would likely contain information related to the objective.

IMPORTANT: You MUST only return URLs that are exactly from the list above. Do not modify, create, or suggest any new URLs.

Consider:
1. URL structure and path names that suggest relevant content
2. Likely page types (product pages, articles, documentation, etc.)
3. Depth and specificity of URLs
4. Avoid redundant or overly similar URLs

Return ONLY a JSON array of the selected URLs that exist in the provided list, nothing else. Example format:
["https://example.com/page1", "https://example.com/page2"]"#,
            domain,
            objective,
            url_list.join("\n"),
            max_urls.min(20) // Conservative limit
        );

        // Call the internal method that returns ClaudeError
        let response = self
            .send_message_internal(&prompt)
            .await
            .map_err(|e| Box::new(e) as LlmError)?;

        let content = response
            .content
            .first()
            .ok_or_else(|| ClaudeError::ApiError("No content in response".to_string()))
            .map_err(|e| Box::new(e) as LlmError)?;

        let selected_urls_from_llm: Vec<String> = serde_json::from_str(&content.text)
            .map_err(|e| ClaudeError::JsonError(e)) // Map serde_json::Error to ClaudeError first
            .map_err(|e| Box::new(e) as LlmError)?;

        // Create a set of valid URLs for fast lookup (using the input `url_list` for validation)
        let valid_urls_set: HashSet<String> = url_list.into_iter().collect();

        // Filter out any URLs that are not in the original list
        let filtered_urls: Vec<String> = selected_urls_from_llm
            .into_iter()
            .filter(|url| {
                let is_valid = valid_urls_set.contains(url);
                if !is_valid {
                    tracing::warn!(
                        "Claude returned URL not in original list, ignoring: {}",
                        url
                    );
                }
                is_valid
            })
            .collect();

        Ok(filtered_urls)
    }

    async fn analyze_content(
        &self,
        objective: &str,
        url: &str,
        content: &str,
    ) -> Result<String, LlmError> {
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
            content.chars().take(8000).collect::<String>()
        );

        let response = self
            .send_message_internal(&prompt)
            .await
            .map_err(|e| Box::new(e) as LlmError)?;

        let content_text = response
            .content
            .first()
            .ok_or_else(|| ClaudeError::ApiError("No content in response".to_string()))
            .map_err(|e| Box::new(e) as LlmError)?
            .text
            .clone();

        Ok(content_text)
    }

    async fn send_message(&self, message: &str) -> Result<ClaudeResponse, LlmError> {
        self.send_message_internal(message)
            .await
            .map_err(|e| Box::new(e) as LlmError)
    }
}
