/// This module provides functionality to generate TypeScript definitions
/// from Rust types using the ts-rs crate and include them in the schema.
use crate::entities::*;
use ts_rs::TS;

/// Generate the complete TypeScript schema by collecting all exported types
pub fn generate_typescript_schema() -> String {
    // Collect all the generated TypeScript definitions
    let mut schema = String::new();

    // Add header comment
    schema.push_str("// TypeScript types generated from Rust entities using ts-rs\n\n");

    // Add all type definitions
    schema.push_str(&Person::decl());
    schema.push('\n');
    schema.push_str(&Location::decl());
    schema.push('\n');
    schema.push_str(&Event::decl());
    schema.push('\n');
    schema.push_str(&EventStatus::decl());
    schema.push('\n');
    schema.push_str(&Price::decl());
    schema.push('\n');
    schema.push_str(&Product::decl());
    schema.push('\n');
    schema.push_str(&ProductAvailability::decl());
    schema.push('\n');
    schema.push_str(&Review::decl());
    schema.push('\n');
    schema.push_str(&Organization::decl());
    schema.push('\n');
    schema.push_str(&ContactInfo::decl());
    schema.push('\n');
    schema.push_str(&NewsArticle::decl());
    schema.push('\n');
    schema.push_str(&JobListing::decl());
    schema.push('\n');
    schema.push_str(&SalaryRange::decl());
    schema.push('\n');
    schema.push_str(&SalaryPeriod::decl());
    schema.push('\n');
    schema.push_str(&EmploymentType::decl());
    schema.push('\n');
    schema.push_str(&ExtractedEntity::decl());
    schema.push('\n');
    schema.push_str(&EntityExtractionResult::decl());
    schema.push('\n');
    schema.push_str(&LLMResponse::decl());
    schema.push('\n');

    // Add response interface for entity extraction
    schema.push_str("\n// Response structure for entity extraction\n");
    schema.push_str("interface EntityExtractionResponse {\n");
    schema.push_str("  entities: ExtractedEntity[];\n");
    schema.push_str("  raw_analysis: string;\n");
    schema.push_str("  extraction_confidence: number; // 0.0 to 1.0\n");
    schema.push_str("}\n");

    schema
}

/// Get the TypeScript schema as a static string
pub fn get_generated_schema() -> &'static str {
    // For now, we'll use a lazy static or generate at runtime
    // In a real implementation, you might want to generate this at build time
    // and include it as a static string
    ""
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_generation() {
        let schema = generate_typescript_schema();
        assert!(!schema.is_empty());
        assert!(schema.contains("type Person"));
        assert!(schema.contains("type Location"));
        assert!(schema.contains("type Event"));
        assert!(schema.contains("type LLMResponse"));
    }
}
