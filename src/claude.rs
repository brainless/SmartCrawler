use crate::llm::{LlmError, LLM}; // Import LLM trait and LlmError
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
// HashSet is no longer needed here as the logic moved to LLM trait default impl
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClaudeError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error), // This might be less used if parsing moves to trait
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Environment variable ANTHROPIC_API_KEY not found")]
    MissingApiKey,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaudeResponse {
    pub id: String,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

    // This internal method is still used by the `send_message` trait method.
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
                "API request failed: {error_text}"
            )));
        }

        let claude_response: ClaudeResponse = response.json().await?;
        Ok(claude_response)
    }
}

#[async_trait]
impl LLM for ClaudeClient {
    // select_urls is now provided by the default implementation in the LLM trait.
    // If ClaudeClient needed a specific version, it would be implemented here.
    // async fn select_urls(
    //     &self,
    //     objective: &str,
    //     urls: &[String],
    //     domain: &str,
    //     max_urls: usize,
    // ) -> Result<Vec<String>, LlmError> {
    //     // Call default trait implementation:
    //     // LLM::select_urls(self, objective, urls, domain, max_urls).await
    //     // Or provide a custom implementation if needed.
    //     // For this refactor, we are removing it to use the default.
    // }

    // analyze_content is now provided by the default implementation in the LLM trait.
    // async fn analyze_content(
    //     &self,
    //     objective: &str,
    //     url: &str,
    //     content: &str,
    // ) -> Result<String, LlmError> {
    //     // Call default trait implementation:
    //     // LLM::analyze_content(self, objective, url, content).await
    //     // Or provide a custom implementation if needed.
    //     // For this refactor, we are removing it to use the default.
    // }

    // This method MUST be implemented by concrete types as it's specific to the LLM provider.
    async fn send_message(&self, message: &str) -> Result<ClaudeResponse, LlmError> {
        self.send_message_internal(message)
            .await
            .map_err(|e| Box::new(e) as LlmError) // Convert ClaudeError to LlmError
    }
}
