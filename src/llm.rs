use crate::claude::ClaudeResponse; // Assuming ClaudeResponse might be generalized later
use crate::entities::{ExtractedEntity, EntityExtractionResult};
use crate::typescript_schema::TYPESCRIPT_SCHEMA;
use async_trait::async_trait;
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

        // Extract path+query for each URL for more focused analysis (improvement from issue #19)
        let url_paths: Vec<String> = url_list
            .iter()
            .enumerate()
            .map(|(i, url)| {
                if let Ok(parsed) = url::Url::parse(url) {
                    let mut path_query = parsed.path().to_string();
                    if let Some(query) = parsed.query() {
                        path_query.push('?');
                        path_query.push_str(query);
                    }
                    format!("{}: {}", i + 1, path_query)
                } else {
                    format!("{}: {}", i + 1, url)
                }
            })
            .collect();

        let prompt = format!(
            r#"You are helping a web crawler select the most relevant URLs to crawl for a specific objective.

Domain: {}
Objective: {}

Here are the available URL paths on this domain (numbered for reference):
{}

Please analyze these URL paths and select the {} most relevant ones that would likely contain information related to the objective.

Focus on:
1. Path structure and directory names that suggest relevant content
2. File names and extensions that indicate relevant page types
3. Query parameters that suggest dynamic content matching the objective
4. Depth and specificity - prefer focused pages over general ones
5. Avoid redundant or overly similar paths

IMPORTANT: Return ONLY the numbers (1, 2, 3, etc.) of the selected paths as a JSON array.
Example format: [1, 3, 7, 12]

Selected path numbers:"#,
            domain,
            objective,
            url_paths.join("\n"),
            max_urls.min(20) // Conservative limit
        );

        let response = self.send_message(&prompt).await?;

        let content_block = response
            .content
            .first()
            .ok_or_else(|| Box::new(DefaultLlmImplError::NoContentInResponse) as LlmError)?;

        // Parse the response as array of numbers (improvement from issue #19)
        let selected_indices: Vec<usize> = serde_json::from_str(&content_block.text)
            .map_err(|e| Box::new(DefaultLlmImplError::JsonParseFailed(e)) as LlmError)?;

        // Convert indices back to URLs
        let filtered_urls: Vec<String> = selected_indices
            .into_iter()
            .filter_map(|index| {
                if index > 0 && index <= url_list.len() {
                    Some(url_list[index - 1].clone()) // Convert 1-based to 0-based indexing
                } else {
                    tracing::warn!("LLM returned invalid index {}, ignoring", index);
                    None
                }
            })
            .collect();

        tracing::info!(
            "LLM selected {} URLs from {} candidates",
            filtered_urls.len(),
            url_list.len()
        );
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
1. Carefully analyze the content to determine if it contains information relevant to the objective
2. If the objective is NOT clearly met by the content, respond with exactly: "OBJECTIVE_NOT_MET"
3. If the objective IS met, extract and present the relevant information using structured formats based on entity types
4. When you find entities, use the TypeScript schema structure below as a guide for organizing the information
5. Present the data in a human-readable format but follow the logical structure of the entities

RESPONSE FORMAT:
- If objective not met: "OBJECTIVE_NOT_MET"
- If objective met: Structured summary using entity-based organization

ENTITY STRUCTURE REFERENCE:
{}

EXAMPLE STRUCTURED RESPONSES:

For "Find people/contact information":
PERSON: John Smith
- Title: CEO
- Company: Tech Corp
- Email: john.doe@techcorp.com
- Phone: +1-555-0123

For "Find events":
EVENT: Tech Conference 2024
- Description: Annual technology conference
- Dates: March 15-17, 2024
- Location: Convention Center, San Francisco, CA
- Registration: https://techconf2024.com
- Status: Upcoming

For "Find products":
PRODUCT: Wireless Headphones
- Brand: TechBrand
- Price: $199.99 USD
- Description: High-quality wireless headphones
- Availability: In Stock

For "Find organizations/companies":
ORGANIZATION: Tech Corp
- Industry: Technology
- Website: https://techcorp.com
- Description: Leading technology company
- Contact: info@techcorp.com

Use this structured approach to present clear, organized information that directly addresses the objective."#,
            url,
            objective,
            content.chars().take(10000).collect::<String>(),
            TYPESCRIPT_SCHEMA
        );

        let response = self.send_message(&prompt).await?;

        let content_text = response
            .content
            .first()
            .ok_or_else(|| Box::new(DefaultLlmImplError::NoContentInResponse) as LlmError)?
            .text
            .clone();

        // Check if the objective was not met and return an error
        if content_text.contains("OBJECTIVE_NOT_MET") {
            return Err(Box::new(DefaultLlmImplError::ObjectiveNotMet(format!(
                "Objective '{}' not clearly met in content from {}",
                objective, url
            ))) as LlmError);
        }

        Ok(content_text)
    }

    /// Extracts structured entities from web content based on the objective.
    async fn extract_entities(
        &self,
        objective: &str,
        url: &str,
        content: &str,
    ) -> Result<EntityExtractionResult, LlmError> {
        let prompt = format!(
            r#"You are analyzing web content to extract structured entities based on a specific objective.

URL: {}
Objective: {}

Content (truncated if necessary):
{}

INSTRUCTIONS:
1. Carefully analyze the content to find information that directly relates to the objective
2. Extract ONLY entities that are clearly present and relevant to the objective
3. Return a JSON object that strictly conforms to the TypeScript schema provided below
4. Ensure all required fields are present and all data types match the schema
5. Use null for optional fields when information is not available

RESPONSE FORMAT:
You must return a JSON object with this exact structure:
{{
  "entities": ExtractedEntity[],
  "raw_analysis": string,
  "extraction_confidence": number // 0.0 to 1.0
}}

TYPESCRIPT SCHEMA:
{}

IMPORTANT GUIDELINES:
- Each entity MUST have a "type" field that exactly matches one of the TypeScript interface names
- Use proper data types: strings for text, numbers for numeric values, booleans for true/false
- For dates, use ISO 8601 format (YYYY-MM-DDTHH:mm:ssZ) or YYYY-MM-DD for date-only fields
- For nested objects (like Location in Event), include the full object structure with type field
- Only extract entities if you have confidence ≥ 0.6 in the accuracy of the extracted data
- If no relevant entities are found, return an empty entities array
- The raw_analysis should briefly describe what entities were found and why

Return ONLY the JSON object, no additional text or explanation."#,
            url,
            objective,
            content.chars().take(12000).collect::<String>(),
            TYPESCRIPT_SCHEMA
        );

        let response = self.send_message(&prompt).await?;

        let content_text = response
            .content
            .first()
            .ok_or_else(|| Box::new(DefaultLlmImplError::NoContentInResponse) as LlmError)?
            .text
            .clone();

        // Parse the JSON response
        let extraction_data: serde_json::Value = serde_json::from_str(&content_text)
            .map_err(|e| Box::new(DefaultLlmImplError::JsonParseFailed(e)) as LlmError)?;

        let mut result = EntityExtractionResult::new(url.to_string(), objective.to_string());
        
        // Extract entities
        if let Some(entities_array) = extraction_data["entities"].as_array() {
            for entity_value in entities_array {
                if let Ok(entity) = serde_json::from_value::<ExtractedEntity>(entity_value.clone()) {
                    result.entities.push(entity);
                }
            }
        }

        // Extract raw analysis
        if let Some(raw_analysis) = extraction_data["raw_analysis"].as_str() {
            result.raw_analysis = raw_analysis.to_string();
        }

        // Extract confidence
        if let Some(confidence) = extraction_data["extraction_confidence"].as_f64() {
            result.extraction_confidence = confidence as f32;
        }

        Ok(result)
    }

    /// Sends a message to the LLM and gets a response.
    /// Note: `ClaudeResponse` is used here. This might need generalization
    /// if other LLMs have significantly different response structures.
    /// This method MUST be implemented by concrete types.
    async fn send_message(&self, message: &str) -> Result<ClaudeResponse, LlmError>;
}
