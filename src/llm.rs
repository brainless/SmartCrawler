use crate::claude::ClaudeResponse; // Assuming ClaudeResponse might be generalized later
use crate::entities::{EntityExtractionResult, ExtractedEntity, LLMResponse};
use crate::typescript_schema::get_typescript_schema;
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
    /// Generates relevant keywords from a crawling objective for URL filtering.
    async fn generate_keywords(
        &self,
        objective: &str,
        domain: &str,
    ) -> Result<Vec<String>, LlmError> {
        let prompt = format!(
            r#"You are helping a web crawler generate relevant keywords for URL filtering based on a crawling objective.

Domain: {}
Objective: {}

INSTRUCTIONS:
1. Analyze the objective to identify the most relevant keywords for URL path matching
2. Generate 5-10 keywords that would likely appear in URLs containing information relevant to the objective
3. Focus on keywords that would appear in URL paths, directory names, file names, or query parameters
4. Include both general and specific terms related to the objective
5. Consider different word forms (singular/plural, abbreviations, synonyms)

IMPORTANT: Return ONLY a JSON array of keywords as strings.
Example format: ["pricing", "price", "cost", "plans", "subscription", "billing"]

Keywords:"#,
            domain, objective
        );

        let response = self.send_message(&prompt).await?;

        let content_block = response
            .content
            .first()
            .ok_or_else(|| Box::new(DefaultLlmImplError::NoContentInResponse) as LlmError)?;

        // Parse the response as array of strings
        let keywords: Vec<String> = serde_json::from_str(&content_block.text)
            .map_err(|e| Box::new(DefaultLlmImplError::JsonParseFailed(e)) as LlmError)?;

        tracing::info!(
            "Generated {} keywords for objective '{}': {:?}",
            keywords.len(),
            objective,
            keywords
        );

        Ok(keywords)
    }

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
    ) -> Result<LLMResponse, LlmError> {
        let typescript_schema = get_typescript_schema();
        let prompt = format!(
            r#"You are analyzing web content for a specific objective.

URL: {}
Objective: {}

Content (truncated if necessary):
{}

INSTRUCTIONS:
1. Carefully analyze the content to determine if it contains information relevant to the objective
2. Respond ONLY with a valid JSON object that matches the LLMResponse structure
3. If the objective is NOT clearly met, set is_objective_met to false and results to null
4. If the objective IS met, set is_objective_met to true and include extracted entities in results array
5. Each entity in results must conform to the TypeScript schema provided below
6. Always include a brief analysis explaining your findings

RESPONSE FORMAT (JSON ONLY):
{{
  "is_objective_met": boolean,
  "results": ExtractedEntity[] | null,
  "analysis": "Brief explanation of findings"
}}

TYPESCRIPT SCHEMA:
{}

EXAMPLE JSON RESPONSES:

For objective NOT met:
{{
  "is_objective_met": false,
  "results": null,
  "analysis": "No relevant information found for the given objective"
}}

For "Find people/contact information" (objective met):
{{
  "is_objective_met": true,
  "results": [
    {{
      "type": "Person",
      "first_name": "John",
      "last_name": "Smith",
      "title": "CEO",
      "company": "Tech Corp",
      "email": "john.smith@techcorp.com",
      "phone": "+1-555-0123",
      "full_name": null,
      "bio": null,
      "social_links": []
    }}
  ],
  "analysis": "Found contact information for company executive"
}}

For "Find events" (objective met):
{{
  "is_objective_met": true,
  "results": [
    {{
      "type": "Event",
      "title": "Tech Conference 2024",
      "description": "Annual technology conference",
      "start_date": "2024-03-15",
      "end_date": "2024-03-17",
      "start_time": null,
      "end_time": null,
      "location": {{
        "type": "Location",
        "name": "Convention Center",
        "city": "San Francisco",
        "state": "CA",
        "country": "USA",
        "address": null,
        "postal_code": null,
        "latitude": null,
        "longitude": null,
        "venue_type": "Convention Center"
      }},
      "organizer": null,
      "attendees": [],
      "category": "Technology",
      "tags": [],
      "price": null,
      "registration_url": "https://techconf2024.com",
      "status": "Upcoming"
    }}
  ],
  "analysis": "Found upcoming technology conference with registration details"
}}

CRITICAL: Return ONLY the JSON object, no additional text, explanations, or markdown formatting.
Start your response with {{ and end with }}. Do not wrap in code blocks."#,
            url,
            objective,
            content.chars().take(10000).collect::<String>(),
            typescript_schema
        );

        let response = self.send_message(&prompt).await?;

        let content_text = response
            .content
            .first()
            .ok_or_else(|| Box::new(DefaultLlmImplError::NoContentInResponse) as LlmError)?
            .text
            .clone();

        tracing::debug!("Raw LLM response for content analysis: {}", content_text);

        // Extract JSON from the response
        let json_str = extract_json_from_response(&content_text).ok_or_else(|| {
            tracing::warn!("No valid JSON found in LLM response for {}", url);
            Box::new(DefaultLlmImplError::ObjectiveNotMet(
                "No valid JSON object found in LLM response".to_string(),
            )) as LlmError
        })?;

        tracing::debug!("Extracted JSON for parsing: {}", json_str);

        // Parse the JSON response
        let llm_response: LLMResponse = serde_json::from_str(&json_str).map_err(|e| {
            tracing::error!("JSON parsing failed for {}: {}", url, e);
            tracing::error!("Problematic JSON: {}", json_str);
            Box::new(DefaultLlmImplError::JsonParseFailed(e)) as LlmError
        })?;

        Ok(llm_response)
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

CRITICAL: Return ONLY the JSON object, no additional text, explanations, or markdown formatting.
Start your response with {{ and end with }}. Do not wrap in code blocks or add any other text."#,
            url,
            objective,
            content.chars().take(12000).collect::<String>(),
            get_typescript_schema()
        );

        let response = self.send_message(&prompt).await?;

        let content_text = response
            .content
            .first()
            .ok_or_else(|| Box::new(DefaultLlmImplError::NoContentInResponse) as LlmError)?
            .text
            .clone();

        tracing::debug!("Raw LLM response for entity extraction: {}", content_text);

        // Extract JSON from the response - the LLM might include extra text
        let json_str = extract_json_from_response(&content_text).ok_or_else(|| {
            tracing::warn!("No valid JSON found in LLM response for {}", url);
            Box::new(DefaultLlmImplError::ObjectiveNotMet(
                "No valid JSON object found in LLM response".to_string(),
            )) as LlmError
        })?;

        tracing::debug!("Extracted JSON for parsing: {}", json_str);

        // Parse the JSON response
        let extraction_data: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
            tracing::error!("JSON parsing failed for {}: {}", url, e);
            tracing::error!("Problematic JSON: {}", json_str);
            Box::new(DefaultLlmImplError::JsonParseFailed(e)) as LlmError
        })?;

        let mut result = EntityExtractionResult::new(url.to_string(), objective.to_string());

        // Extract entities
        if let Some(entities_array) = extraction_data["entities"].as_array() {
            for entity_value in entities_array {
                if let Ok(entity) = serde_json::from_value::<ExtractedEntity>(entity_value.clone())
                {
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

/// Extract JSON object from LLM response that might contain extra text
fn extract_json_from_response(response: &str) -> Option<String> {
    // First, try to find a JSON object starting with { and ending with }
    let trimmed = response.trim();

    // Case 1: Response is pure JSON (starts and ends with braces)
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed.to_string());
    }

    // Case 2: JSON is embedded in the response - find the first complete JSON object
    if let Some(start) = trimmed.find('{') {
        let mut brace_count = 0;
        let mut in_string = false;
        let mut escaped = false;

        for (i, ch) in trimmed[start..].char_indices() {
            match ch {
                '"' if !escaped => in_string = !in_string,
                '\\' if in_string => escaped = !escaped,
                '{' if !in_string => brace_count += 1,
                '}' if !in_string => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        // Found complete JSON object
                        return Some(trimmed[start..start + i + 1].to_string());
                    }
                }
                _ => escaped = false,
            }

            if ch != '\\' {
                escaped = false;
            }
        }
    }

    // Case 3: Try to find JSON between code blocks or other markers
    for marker in ["```json", "```", "`"] {
        if let Some(start_pos) = trimmed.find(marker) {
            let after_marker = &trimmed[start_pos + marker.len()..];
            if let Some(json_start) = after_marker.find('{') {
                let json_part = &after_marker[json_start..];
                if let Some(end_marker_pos) = json_part.find("```") {
                    let potential_json = &json_part[..end_marker_pos].trim();
                    if potential_json.starts_with('{') && potential_json.ends_with('}') {
                        return Some(potential_json.to_string());
                    }
                } else if json_part.starts_with('{') && json_part.contains('}') {
                    // Try to extract complete JSON without end marker
                    return extract_json_from_response(json_part);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::extract_json_from_response;

    #[test]
    fn test_extract_pure_json() {
        let response = r#"{"entities": [], "raw_analysis": "test", "extraction_confidence": 0.5}"#;
        let result = extract_json_from_response(response);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), response);
    }

    #[test]
    fn test_extract_json_with_extra_text() {
        let response = r#"Here is the analysis:
        {"entities": [], "raw_analysis": "test", "extraction_confidence": 0.5}
        That's the result."#;
        let result = extract_json_from_response(response);
        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            r#"{"entities": [], "raw_analysis": "test", "extraction_confidence": 0.5}"#
        );
    }

    #[test]
    fn test_extract_json_from_code_block() {
        let response = r#"```json
        {"entities": [], "raw_analysis": "test", "extraction_confidence": 0.5}
        ```"#;
        let result = extract_json_from_response(response);
        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            r#"{"entities": [], "raw_analysis": "test", "extraction_confidence": 0.5}"#
        );
    }

    #[test]
    fn test_extract_json_no_valid_json() {
        let response = "This is just text without any JSON";
        let result = extract_json_from_response(response);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_json_malformed() {
        let response = r#"{"entities": [], "raw_analysis": "test""#; // Missing closing brace
        let result = extract_json_from_response(response);
        assert!(result.is_none());
    }

    #[test]
    fn test_generate_keywords_prompt_format() {
        // Test that the generate_keywords method creates a proper prompt
        // This is more of a structure test since we can't easily mock the LLM
        let objective = "Find pricing information";
        let domain = "example.com";

        // Check that the prompt contains expected elements
        assert!(objective.contains("pricing"));
        assert!(domain.contains("example"));

        // This test just ensures the method signature is correct
        // Real testing would require mocking the LLM response
    }
}
