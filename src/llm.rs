use crate::claude::ClaudeResponse; // Assuming ClaudeResponse might be generalized later
use async_trait::async_trait;
use std::collections::HashSet; // Added for default select_urls implementation
use thiserror::Error; // Added for custom errors within default impls

/// Generic error type for LLM operations.
pub type LlmError = Box<dyn std::error::Error + Send + Sync>;

// Define specific errors that can occur within the default implementations
#[derive(Error, Debug)]
enum DefaultLlmImplError {
    #[error("No content in LLM response")]
    NoContentInResponse,
    #[error("JSON parsing failed: {0}")]
    JsonParseFailed(#[from] serde_json::Error),
    #[error("Objective not met: {0}")]
    ObjectiveNotMet(String),
}

#[async_trait]
pub trait LLM {
    /// Selects the most relevant URLs from a given list based on an objective.
    async fn select_urls(
        &self,
        objective: &str,
        urls: &[String],
        domain: &str,
        max_urls: usize,
    ) -> Result<Vec<String>, LlmError> {
        // url_list is now directly from the `urls` parameter.
        // We clone it to own the data and be able to call .join() later.
        // Also apply the take(200) limit like before.
        let url_list: Vec<String> = urls.iter().take(200).cloned().collect();

        tracing::info!("URLs provided to LLM for selection: {:?}", url_list);

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

        let response = self.send_message(&prompt).await?;

        let content_block = response
            .content
            .first()
            .ok_or_else(|| Box::new(DefaultLlmImplError::NoContentInResponse) as LlmError)?;

        let selected_urls_from_llm: Vec<String> = serde_json::from_str(&content_block.text)
            .map_err(|e| Box::new(DefaultLlmImplError::JsonParseFailed(e)) as LlmError)?;

        // Create a set of valid URLs for fast lookup (using the input `url_list` for validation)
        let valid_urls_set: HashSet<String> = url_list.into_iter().collect();

        // Filter out any URLs that are not in the original list
        let filtered_urls: Vec<String> = selected_urls_from_llm
            .into_iter()
            .filter(|url| {
                let is_valid = valid_urls_set.contains(url);
                if !is_valid {
                    tracing::warn!(
                        "LLM returned URL not in original list, ignoring: {}",
                        url
                    );
                }
                is_valid
            })
            .collect();

        Ok(filtered_urls)
    }

    /// Analyzes web content for a specific objective.
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

INSTRUCTIONS:
1. First, determine if this page contains information that directly relates to the objective
2. If the objective is NOT clearly met by the content, respond with exactly: "OBJECTIVE_NOT_MET"
3. If the objective IS met, extract and return ONLY the specific information relevant to the objective
4. Do not provide key findings, actionable insights, or additional analysis - only the relevant information itself

Response format:
- If objective not met: "OBJECTIVE_NOT_MET"
- If objective met: Only the relevant information from the content"#,
            url,
            objective,
            content.chars().take(8000).collect::<String>()
        );

        let response = self.send_message(&prompt).await?;

        let content_text = response
            .content
            .first()
            .ok_or_else(|| Box::new(DefaultLlmImplError::NoContentInResponse) as LlmError)?
            .text
            .clone();

        // Check if the objective was not met and return an error
        if content_text.trim() == "OBJECTIVE_NOT_MET" {
            return Err(Box::new(DefaultLlmImplError::ObjectiveNotMet(format!(
                "Objective '{}' not clearly met in content from {}",
                objective, url
            ))) as LlmError);
        }

        Ok(content_text)
    }

    /// Sends a message to the LLM and gets a response.
    /// Note: `ClaudeResponse` is used here. This might need generalization
    /// if other LLMs have significantly different response structures.
    /// This method MUST be implemented by concrete types.
    async fn send_message(&self, message: &str) -> Result<ClaudeResponse, LlmError>;
}
