use crate::claude::ClaudeResponse; // Assuming ClaudeResponse might be generalized later
use async_trait::async_trait;

/// Generic error type for LLM operations.
pub type LlmError = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
pub trait LLM {
    /// Selects the most relevant URLs from a given list based on an objective.
    // Changed urls: &[T] to urls: &[String] to make the trait object-safe (dyn-compatible)
    async fn select_urls(
        &self,
        objective: &str,
        urls: &[String], // Changed from generic &[T]
        domain: &str,
        max_urls: usize,
    ) -> Result<Vec<String>, LlmError>;

    /// Analyzes web content for a specific objective.
    async fn analyze_content(
        &self,
        objective: &str,
        url: &str,
        content: &str,
    ) -> Result<String, LlmError>;

    /// Sends a message to the LLM and gets a response.
    /// Note: `ClaudeResponse` is used here. This might need generalization
    /// if other LLMs have significantly different response structures.
    async fn send_message(
        &self,
        message: &str,
    ) -> Result<ClaudeResponse, LlmError>;
}
