use regex::Regex;
use std::collections::HashMap;

/// Template variable types that can be detected
#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    Number, // Integer numbers
    Float,  // Floating point numbers
}

/// Represents a template pattern with variable placeholders
#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    pub pattern: String, // The template pattern like "{count} comments"
    pub variables: Vec<(String, VariableType)>, // Variable names and their types
}

/// Template detector that can identify common patterns in text
pub struct TemplateDetector {
    // Common time unit patterns
    time_units: HashMap<String, String>,
    // Common count/quantity descriptors
    count_descriptors: HashMap<String, String>,
    // Regex patterns for detection
    number_regex: Regex,
    float_regex: Regex,
}

impl TemplateDetector {
    pub fn new() -> Self {
        let mut time_units = HashMap::new();
        time_units.insert("second".to_string(), "time".to_string());
        time_units.insert("seconds".to_string(), "time".to_string());
        time_units.insert("minute".to_string(), "time".to_string());
        time_units.insert("minutes".to_string(), "time".to_string());
        time_units.insert("hour".to_string(), "time".to_string());
        time_units.insert("hours".to_string(), "time".to_string());
        time_units.insert("day".to_string(), "time".to_string());
        time_units.insert("days".to_string(), "time".to_string());
        time_units.insert("week".to_string(), "time".to_string());
        time_units.insert("weeks".to_string(), "time".to_string());
        time_units.insert("month".to_string(), "time".to_string());
        time_units.insert("months".to_string(), "time".to_string());
        time_units.insert("year".to_string(), "time".to_string());
        time_units.insert("years".to_string(), "time".to_string());

        let mut count_descriptors = HashMap::new();
        count_descriptors.insert("comment".to_string(), "count".to_string());
        count_descriptors.insert("comments".to_string(), "count".to_string());
        count_descriptors.insert("reply".to_string(), "count".to_string());
        count_descriptors.insert("replies".to_string(), "count".to_string());
        count_descriptors.insert("like".to_string(), "count".to_string());
        count_descriptors.insert("likes".to_string(), "count".to_string());
        count_descriptors.insert("view".to_string(), "count".to_string());
        count_descriptors.insert("views".to_string(), "count".to_string());
        count_descriptors.insert("share".to_string(), "count".to_string());
        count_descriptors.insert("shares".to_string(), "count".to_string());
        count_descriptors.insert("point".to_string(), "count".to_string());
        count_descriptors.insert("points".to_string(), "count".to_string());
        count_descriptors.insert("upvote".to_string(), "count".to_string());
        count_descriptors.insert("upvotes".to_string(), "count".to_string());
        count_descriptors.insert("item".to_string(), "count".to_string());
        count_descriptors.insert("items".to_string(), "count".to_string());

        let number_regex = Regex::new(r"\b\d+\b").unwrap();
        let float_regex = Regex::new(r"\b\d+\.\d+\b").unwrap();

        TemplateDetector {
            time_units,
            count_descriptors,
            number_regex,
            float_regex,
        }
    }

    /// Detect template pattern in given text content
    pub fn detect_template(&self, content: &str) -> Option<Template> {
        let content = content.trim();
        if content.is_empty() {
            return None;
        }

        // First try to detect float patterns, then number patterns
        if let Some(template) = self.detect_float_pattern(content) {
            return Some(template);
        }

        if let Some(template) = self.detect_number_pattern(content) {
            return Some(template);
        }

        None
    }

    /// Detect patterns with floating point numbers
    fn detect_float_pattern(&self, content: &str) -> Option<Template> {
        let float_matches: Vec<_> = self.float_regex.find_iter(content).collect();
        if float_matches.is_empty() {
            return None;
        }

        for (i, float_match) in float_matches.iter().enumerate() {
            let var_name = format!(
                "value{}",
                if i == 0 {
                    "".to_string()
                } else {
                    i.to_string()
                }
            );

            // Replace only this specific float occurrence with placeholder
            let mut pattern_content = content.to_string();
            let start = float_match.start();
            let end = float_match.end();
            pattern_content.replace_range(start..end, &format!("{{{var_name}}}"));

            if self.is_valid_pattern(&pattern_content) {
                return Some(Template {
                    pattern: pattern_content,
                    variables: vec![(var_name, VariableType::Float)],
                });
            }
        }

        None
    }

    /// Detect patterns with integer numbers
    fn detect_number_pattern(&self, content: &str) -> Option<Template> {
        let number_matches: Vec<_> = self.number_regex.find_iter(content).collect();
        if number_matches.is_empty() {
            return None;
        }

        // Try each number match individually
        for (i, number_match) in number_matches.iter().enumerate() {
            // Determine appropriate variable name based on context
            let var_name = self.determine_variable_name(content, number_match.start(), i);

            // Replace only this specific number occurrence with placeholder
            let mut pattern_content = content.to_string();
            let start = number_match.start();
            let end = number_match.end();
            pattern_content.replace_range(start..end, &format!("{{{var_name}}}"));

            if self.is_valid_pattern(&pattern_content) {
                return Some(Template {
                    pattern: pattern_content,
                    variables: vec![(var_name, VariableType::Number)],
                });
            }
        }

        None
    }

    /// Determine appropriate variable name based on context around the number
    fn determine_variable_name(&self, content: &str, number_pos: usize, index: usize) -> String {
        let words: Vec<&str> = content.split_whitespace().collect();

        // Find the number in the word sequence
        let mut current_pos = 0;
        for (word_idx, word) in words.iter().enumerate() {
            if current_pos <= number_pos && number_pos < current_pos + word.len() {
                // Check next word for context
                if word_idx + 1 < words.len() {
                    let next_word = words[word_idx + 1].to_lowercase();

                    // Check for time units
                    if self.time_units.contains_key(&next_word) {
                        return "time".to_string();
                    }

                    // Check for count descriptors
                    if self.count_descriptors.contains_key(&next_word) {
                        return "count".to_string();
                    }

                    // Check for "ago" pattern
                    if word_idx + 2 < words.len() && words[word_idx + 2].to_lowercase() == "ago" {
                        return "time".to_string();
                    }
                }

                // Check previous word for context
                if word_idx > 0 {
                    let prev_word = words[word_idx - 1].to_lowercase();
                    if prev_word == "page" || prev_word == "item" {
                        return "count".to_string();
                    }
                }

                break;
            }
            current_pos += word.len() + 1; // +1 for space
        }

        // Default naming
        format!(
            "value{}",
            if index == 0 {
                "".to_string()
            } else {
                index.to_string()
            }
        )
    }

    /// Check if the pattern contains recognizable template elements
    fn is_valid_pattern(&self, pattern: &str) -> bool {
        let words: Vec<&str> = pattern.split_whitespace().collect();

        // Must have at least one placeholder
        if !pattern.contains('{') || !pattern.contains('}') {
            return false;
        }

        // Must have at least 2 words (placeholder + descriptor)
        if words.len() < 2 {
            return false;
        }

        // Check for known patterns
        for word in &words {
            let lowercase = word.to_lowercase();
            let clean_word = lowercase.trim_matches(|c: char| !c.is_alphabetic());

            // Time units
            if self.time_units.contains_key(clean_word) {
                return true;
            }

            // Count descriptors
            if self.count_descriptors.contains_key(clean_word) {
                return true;
            }

            // Common template indicators
            if clean_word == "ago" || clean_word == "per" || clean_word == "of" {
                return true;
            }
        }

        // Don't accept random patterns without recognizable indicators
        false
    }

    /// Apply template to content, returning the template version if applicable
    pub fn apply_template(&self, content: &str) -> String {
        if let Some(template) = self.detect_template(content) {
            template.pattern
        } else {
            content.to_string()
        }
    }
}

impl Default for TemplateDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_comment_pattern() {
        let detector = TemplateDetector::new();

        let template = detector.detect_template("42 comments").unwrap();
        assert_eq!(template.pattern, "{count} comments");
        assert_eq!(template.variables.len(), 1);
        assert_eq!(template.variables[0].0, "count");
        assert_eq!(template.variables[0].1, VariableType::Number);
    }

    #[test]
    fn test_time_ago_pattern() {
        let detector = TemplateDetector::new();

        let template = detector.detect_template("16 hours ago").unwrap();
        assert_eq!(template.pattern, "{time} hours ago");
        assert_eq!(template.variables.len(), 1);
        assert_eq!(template.variables[0].0, "time");
        assert_eq!(template.variables[0].1, VariableType::Number);
    }

    #[test]
    fn test_plural_time_units() {
        let detector = TemplateDetector::new();

        let template = detector.detect_template("1 minute ago").unwrap();
        assert_eq!(template.pattern, "{time} minute ago");

        let template = detector.detect_template("5 minutes ago").unwrap();
        assert_eq!(template.pattern, "{time} minutes ago");
    }

    #[test]
    fn test_float_pattern() {
        let detector = TemplateDetector::new();

        let template = detector.detect_template("4.5 hours ago").unwrap();
        assert_eq!(template.pattern, "{value} hours ago");
        assert_eq!(template.variables[0].1, VariableType::Float);
    }

    #[test]
    fn test_various_count_descriptors() {
        let detector = TemplateDetector::new();

        let patterns = vec![
            ("123 likes", "{count} likes"),
            ("42 views", "{count} views"),
            ("7 replies", "{count} replies"),
            ("1 share", "{count} share"),
            ("999 points", "{count} points"),
        ];

        for (input, expected) in patterns {
            let template = detector.detect_template(input).unwrap();
            assert_eq!(template.pattern, expected, "Failed for input: {}", input);
            assert_eq!(template.variables[0].1, VariableType::Number);
        }
    }

    #[test]
    fn test_various_time_units() {
        let detector = TemplateDetector::new();

        let patterns = vec![
            ("30 seconds ago", "{time} seconds ago"),
            ("2 days ago", "{time} days ago"),
            ("1 week ago", "{time} week ago"),
            ("6 months ago", "{time} months ago"),
            ("2 years ago", "{time} years ago"),
        ];

        for (input, expected) in patterns {
            let template = detector.detect_template(input).unwrap();
            assert_eq!(template.pattern, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_no_pattern_detection() {
        let detector = TemplateDetector::new();

        let non_patterns = vec![
            "Hello world",
            "Just text",
            "42",              // Just a number
            "Random 123 text", // Number without recognizable pattern
            "",
        ];

        for input in non_patterns {
            assert!(
                detector.detect_template(input).is_none(),
                "Should not detect pattern for: {}",
                input
            );
        }
    }

    #[test]
    fn test_apply_template() {
        let detector = TemplateDetector::new();

        assert_eq!(detector.apply_template("42 comments"), "{count} comments");
        assert_eq!(detector.apply_template("16 hours ago"), "{time} hours ago");
        assert_eq!(detector.apply_template("Hello world"), "Hello world");
    }

    #[test]
    fn test_edge_cases() {
        let detector = TemplateDetector::new();

        // Multiple numbers - should pick the first one that makes sense
        let template = detector
            .detect_template("Posted 2 hours ago by user123")
            .unwrap();
        assert_eq!(template.pattern, "Posted {time} hours ago by user123");

        // Complex patterns
        let template = detector.detect_template("Page 5 of 100").unwrap();
        assert_eq!(template.pattern, "Page {count} of 100");
    }

    #[test]
    fn test_case_insensitive_matching() {
        let detector = TemplateDetector::new();

        let template = detector.detect_template("42 COMMENTS").unwrap();
        assert_eq!(template.pattern, "{count} COMMENTS");

        let template = detector.detect_template("16 Hours Ago").unwrap();
        assert_eq!(template.pattern, "{time} Hours Ago");
    }

    #[test]
    fn test_whitespace_handling() {
        let detector = TemplateDetector::new();

        let template = detector.detect_template("  42   comments  ").unwrap();
        assert_eq!(template.pattern, "{count}   comments");

        let template = detector.detect_template("16\thours\tago").unwrap();
        assert_eq!(template.pattern, "{time}\thours\tago");
    }

    #[test]
    fn test_integration_with_html_parsing() {
        use crate::html_parser::HtmlParser;

        let detector = TemplateDetector::new();
        let parser = HtmlParser::new();

        // Test with HTML that contains template-detectable content
        let html = r#"<html><body>
            <div class="comment-count">42 comments</div>
            <div class="timestamp">16 hours ago</div>
            <div class="views">1.2k views</div>
            <div class="rating">4.5 stars</div>
            <div class="other">Just some text</div>
        </body></html>"#;

        let tree = parser.parse(html);

        // Find content nodes and test template detection
        let body = &tree.children[0]; // body

        let comment_count = &body.children[0]; // first div
        assert_eq!(comment_count.content, "42 comments");
        assert_eq!(
            detector.apply_template(&comment_count.content),
            "{count} comments"
        );

        let timestamp = &body.children[1]; // second div
        assert_eq!(timestamp.content, "16 hours ago");
        assert_eq!(
            detector.apply_template(&timestamp.content),
            "{time} hours ago"
        );

        let other = &body.children[4]; // fifth div
        assert_eq!(other.content, "Just some text");
        assert_eq!(detector.apply_template(&other.content), "Just some text"); // no template
    }

    #[test]
    fn test_social_media_patterns() {
        let detector = TemplateDetector::new();

        let social_patterns = vec![
            ("999 likes", "{count} likes"),
            ("1.2k views", "{count}.2k views"), // Detected as number + .2k
            ("42 shares", "{count} shares"),
            ("10 upvotes", "{count} upvotes"),
            ("500 points", "{count} points"),
        ];

        for (input, expected) in social_patterns {
            let result = detector.apply_template(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_time_patterns_comprehensive() {
        let detector = TemplateDetector::new();

        let time_patterns = vec![
            ("just now", "just now"), // No template
            ("1 second ago", "{time} second ago"),
            ("30 seconds ago", "{time} seconds ago"),
            ("2 minutes ago", "{time} minutes ago"),
            ("1 hour ago", "{time} hour ago"),
            ("5 hours ago", "{time} hours ago"),
            ("yesterday", "yesterday"), // No template
            ("2 days ago", "{time} days ago"),
            ("1 week ago", "{time} week ago"),
            ("3 weeks ago", "{time} weeks ago"),
            ("last month", "last month"), // No template
            ("2 months ago", "{time} months ago"),
            ("1 year ago", "{time} year ago"),
        ];

        for (input, expected) in time_patterns {
            let result = detector.apply_template(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_template_based_duplicate_detection() {
        use crate::html_parser::HtmlParser;
        use crate::storage::{FetchStatus, UrlStorage};

        let mut storage = UrlStorage::new();
        let parser = HtmlParser::new();
        let detector = TemplateDetector::new();

        // Add URLs to storage
        storage.add_url("https://example.com/page1".to_string());
        storage.add_url("https://example.com/page2".to_string());

        // Create HTML with similar structures but different values
        let html1 = r#"<html><body>
            <div class="comments">42 comments</div>
            <div class="timestamp">2 hours ago</div>
            <div class="likes">123 likes</div>
        </body></html>"#;

        let html2 = r#"<html><body>
            <div class="comments">16 comments</div>
            <div class="timestamp">5 hours ago</div>
            <div class="likes">89 likes</div>
        </body></html>"#;

        let mut tree1 = parser.parse(html1);
        let mut tree2 = parser.parse(html2);

        // Apply template detection to the trees
        apply_template_to_tree(&mut tree1, &detector);
        apply_template_to_tree(&mut tree2, &detector);

        // Set the HTML data for both URLs
        if let Some(url_data) = storage.get_url_data_mut("https://example.com/page1") {
            url_data.set_html_data(html1.to_string(), tree1, Some("Page 1".to_string()));
            url_data.update_status(FetchStatus::Success);
        }

        if let Some(url_data) = storage.get_url_data_mut("https://example.com/page2") {
            url_data.set_html_data(html2.to_string(), tree2, Some("Page 2".to_string()));
            url_data.update_status(FetchStatus::Success);
        }

        // Analyze domain duplicates after template detection
        storage.analyze_domain_duplicates("example.com");

        let duplicates = storage.get_domain_duplicates("example.com");
        assert!(duplicates.is_some());

        let duplicates = duplicates.unwrap();

        // All three content patterns should be detected as duplicates now:
        // "42 comments" and "16 comments" both become "{count} comments"
        // "2 hours ago" and "5 hours ago" both become "{time} hours ago"
        // "123 likes" and "89 likes" both become "{count} likes"
        assert!(duplicates.get_duplicate_count() > 0);

        // Verify that the template-converted content is considered duplicate
        let page1_tree = storage
            .get_url_data("https://example.com/page1")
            .and_then(|data| data.html_tree.as_ref())
            .unwrap();
        let body = &page1_tree.children[0];

        // Check that content has been converted to templates
        assert_eq!(body.children[0].content, "{count} comments");
        assert_eq!(body.children[1].content, "{time} hours ago");
        assert_eq!(body.children[2].content, "{count} likes");
    }

    /// Helper function to apply template detection to an HTML tree (for testing)
    fn apply_template_to_tree(
        node: &mut crate::html_parser::HtmlNode,
        detector: &TemplateDetector,
    ) {
        if !node.content.is_empty() {
            node.content = detector.apply_template(&node.content);
        }

        for child in &mut node.children {
            apply_template_to_tree(child, detector);
        }
    }

    #[test]
    fn test_template_mode_without_duplicate_filtering() {
        use crate::html_parser::HtmlParser;
        use crate::storage::{FetchStatus, UrlStorage};

        let mut storage = UrlStorage::new();
        let parser = HtmlParser::new();
        let detector = TemplateDetector::new();

        // Add URLs to storage
        storage.add_url("https://example.com/page1".to_string());
        storage.add_url("https://example.com/page2".to_string());

        // Create HTML with template-detectable content
        let html1 = r#"<html><body>
            <div class="comments">42 comments</div>
            <div class="timestamp">2 hours ago</div>
        </body></html>"#;

        let html2 = r#"<html><body>
            <div class="comments">16 comments</div>
            <div class="timestamp">5 hours ago</div>
        </body></html>"#;

        let mut tree1 = parser.parse(html1);
        let mut tree2 = parser.parse(html2);

        // Apply template detection to the trees (simulating template mode)
        apply_template_to_tree(&mut tree1, &detector);
        apply_template_to_tree(&mut tree2, &detector);

        // Set the HTML data for both URLs
        if let Some(url_data) = storage.get_url_data_mut("https://example.com/page1") {
            url_data.set_html_data(html1.to_string(), tree1, Some("Page 1".to_string()));
            url_data.update_status(FetchStatus::Success);
        }

        if let Some(url_data) = storage.get_url_data_mut("https://example.com/page2") {
            url_data.set_html_data(html2.to_string(), tree2, Some("Page 2".to_string()));
            url_data.update_status(FetchStatus::Success);
        }

        // In template mode, we should NOT analyze domain duplicates
        // So let's verify that without calling analyze_domain_duplicates,
        // we get no duplicate information
        let duplicates = storage.get_domain_duplicates("example.com");
        assert!(duplicates.is_none(), "No duplicates should be analyzed in template mode");

        // Verify that content has been converted to templates and is visible
        let page1_tree = storage
            .get_url_data("https://example.com/page1")
            .and_then(|data| data.html_tree.as_ref())
            .unwrap();
        let page2_tree = storage
            .get_url_data("https://example.com/page2")
            .and_then(|data| data.html_tree.as_ref())
            .unwrap();

        // Both pages should show template patterns, not "[FILTERED DUPLICATE]"
        let body1 = &page1_tree.children[0];
        let body2 = &page2_tree.children[0];

        assert_eq!(body1.children[0].content, "{count} comments");
        assert_eq!(body1.children[1].content, "{time} hours ago");
        assert_eq!(body2.children[0].content, "{count} comments");
        assert_eq!(body2.children[1].content, "{time} hours ago");

        // Verify that both pages show the same template patterns
        // (which demonstrates the value of template detection)
        assert_eq!(body1.children[0].content, body2.children[0].content);
        assert_eq!(body1.children[1].content, body2.children[1].content);
    }
}
