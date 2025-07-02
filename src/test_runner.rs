use crate::{
    claude::ClaudeClient, cli::CrawlerConfig, crawler::SmartCrawler,
    entities::EntityExtractionResult, url_ranking::UrlRankingConfig,
};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
};
use tokio::fs;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub description: String,
    pub url: String,
    pub objective: String,
    pub expected_entities: ExpectedEntities,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExpectedEntities {
    pub persons: Option<Vec<ExpectedPerson>>,
    pub locations: Option<Vec<ExpectedLocation>>,
    pub events: Option<Vec<ExpectedEvent>>,
    pub products: Option<Vec<ExpectedProduct>>,
    pub organizations: Option<Vec<ExpectedOrganization>>,
    pub minimum_entities_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedPerson {
    pub name_contains: Option<String>,
    pub title_contains: Option<String>,
    pub company_contains: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedLocation {
    pub name_contains: Option<String>,
    pub city_contains: Option<String>,
    pub country_contains: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedEvent {
    pub title_contains: Option<String>,
    pub location_contains: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedProduct {
    pub name_contains: Option<String>,
    pub brand_contains: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedOrganization {
    pub name_contains: Option<String>,
    pub industry_contains: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub extracted_entities_count: usize,
    pub expected_entities_count: usize,
    pub execution_time_ms: u64,
    pub analysis_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub tests: Vec<TestCase>,
}

pub struct TestRunner {
    client: Arc<ClaudeClient>,
}

impl TestRunner {
    pub fn new(client: Arc<ClaudeClient>) -> Self {
        Self { client }
    }

    /// Load test cases from a JSON file
    pub async fn load_test_suite<P: AsRef<Path>>(
        path: P,
    ) -> Result<TestSuite, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path).await?;
        let test_suite: TestSuite = serde_json::from_str(&content)?;
        Ok(test_suite)
    }

    /// Run all tests in the test suite
    pub async fn run_test_suite(&self, test_suite: TestSuite) -> Vec<TestResult> {
        let mut results = Vec::new();

        for test_case in test_suite.tests {
            info!("Running test: {}", test_case.name);
            let result = self.run_single_test(test_case).await;
            results.push(result);
        }

        results
    }

    /// Run a single test case
    pub async fn run_single_test(&self, test_case: TestCase) -> TestResult {
        let start_time = std::time::Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Create crawler configuration for this test
        let url_ranking_config = UrlRankingConfig::default();
        let config = CrawlerConfig {
            objective: test_case.objective.clone(),
            domains: Arc::new(Mutex::new(vec![])),
            links: Some(vec![test_case.url.clone()]),
            max_urls_per_domain: 1, // Only test the single provided URL
            delay_ms: 0,
            output_file: None,
            verbose: false,
            url_ranking_config,
            enable_keyword_filtering: false,
        };

        // Create crawler and run it
        let crawler = match SmartCrawler::new(config.clone(), self.client.clone()).await {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("Failed to create crawler: {}", e));
                return TestResult {
                    test_name: test_case.name,
                    passed: false,
                    errors,
                    warnings,
                    extracted_entities_count: 0,
                    expected_entities_count: self
                        .count_expected_entities(&test_case.expected_entities),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    analysis_summary: String::new(),
                };
            }
        };

        let mut all_extraction_results = Vec::new();
        let mut analysis_summary = String::new();

        match crawler.crawl_all_domains().await {
            Ok(results) => {
                for result in results.results {
                    all_extraction_results.extend(result.extracted_entities);
                    for analysis in result.analysis {
                        if !analysis_summary.is_empty() {
                            analysis_summary.push_str("\n\n");
                        }
                        analysis_summary.push_str(&analysis);
                    }
                }
            }
            Err(e) => {
                errors.push(format!("Crawler execution failed: {}", e));
            }
        }

        // Validate results against expectations
        let validation_result =
            self.validate_extraction_results(&test_case.expected_entities, &all_extraction_results);
        errors.extend(validation_result.errors);
        warnings.extend(validation_result.warnings);

        let execution_time = start_time.elapsed().as_millis() as u64;
        let passed = errors.is_empty();

        let total_entities = all_extraction_results
            .iter()
            .map(|r| r.entity_count())
            .sum();

        TestResult {
            test_name: test_case.name,
            passed,
            errors,
            warnings,
            extracted_entities_count: total_entities,
            expected_entities_count: self.count_expected_entities(&test_case.expected_entities),
            execution_time_ms: execution_time,
            analysis_summary,
        }
    }

    /// Validate extracted entities against expected results
    fn validate_extraction_results(
        &self,
        expected: &ExpectedEntities,
        actual_results: &[EntityExtractionResult],
    ) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Aggregate all entities from all results
        let all_persons: Vec<&crate::entities::Person> = actual_results
            .iter()
            .flat_map(|r| r.get_persons())
            .collect();
        let all_locations: Vec<&crate::entities::Location> = actual_results
            .iter()
            .flat_map(|r| r.get_locations())
            .collect();

        let total_entities: usize = actual_results.iter().map(|r| r.entity_count()).sum();

        // Check minimum entities count
        if let Some(min_count) = expected.minimum_entities_count {
            if total_entities < min_count {
                errors.push(format!(
                    "Expected minimum {} entities, but got {}",
                    min_count, total_entities
                ));
            }
        }

        // Validate persons
        if let Some(expected_persons) = &expected.persons {
            for expected_person in expected_persons {
                let found = self.find_matching_person(expected_person, &all_persons);
                if !found {
                    errors.push(format!(
                        "Expected person not found: name_contains={:?}, title_contains={:?}, company_contains={:?}",
                        expected_person.name_contains,
                        expected_person.title_contains,
                        expected_person.company_contains
                    ));
                }
            }
        }

        // Validate locations
        if let Some(expected_locations) = &expected.locations {
            for expected_location in expected_locations {
                let found = self.find_matching_location(expected_location, &all_locations);
                if !found {
                    errors.push(format!(
                        "Expected location not found: name_contains={:?}, city_contains={:?}, country_contains={:?}",
                        expected_location.name_contains,
                        expected_location.city_contains,
                        expected_location.country_contains
                    ));
                }
            }
        }

        // Add warnings for low confidence extractions
        let avg_confidence: f32 = if actual_results.is_empty() {
            0.0
        } else {
            actual_results
                .iter()
                .map(|r| r.extraction_confidence)
                .sum::<f32>()
                / actual_results.len() as f32
        };

        if avg_confidence < 0.5 {
            warnings.push(format!(
                "Low confidence extraction: {:.1}%. Consider improving the objective or checking the URL content.",
                avg_confidence * 100.0
            ));
        }

        ValidationResult { errors, warnings }
    }

    fn find_matching_person(
        &self,
        expected: &ExpectedPerson,
        actual_persons: &[&crate::entities::Person],
    ) -> bool {
        for person in actual_persons {
            let mut matches = true;

            if let Some(name_contains) = &expected.name_contains {
                let display_name = person.display_name().to_lowercase();
                if !display_name.contains(&name_contains.to_lowercase()) {
                    matches = false;
                }
            }

            if let Some(title_contains) = &expected.title_contains {
                if let Some(title) = &person.title {
                    if !title
                        .to_lowercase()
                        .contains(&title_contains.to_lowercase())
                    {
                        matches = false;
                    }
                } else {
                    matches = false;
                }
            }

            if let Some(company_contains) = &expected.company_contains {
                if let Some(company) = &person.company {
                    if !company
                        .to_lowercase()
                        .contains(&company_contains.to_lowercase())
                    {
                        matches = false;
                    }
                } else {
                    matches = false;
                }
            }

            if matches {
                return true;
            }
        }
        false
    }

    fn find_matching_location(
        &self,
        expected: &ExpectedLocation,
        actual_locations: &[&crate::entities::Location],
    ) -> bool {
        for location in actual_locations {
            let mut matches = true;

            if let Some(name_contains) = &expected.name_contains {
                if let Some(name) = &location.name {
                    if !name.to_lowercase().contains(&name_contains.to_lowercase()) {
                        matches = false;
                    }
                } else {
                    matches = false;
                }
            }

            if let Some(city_contains) = &expected.city_contains {
                if let Some(city) = &location.city {
                    if !city.to_lowercase().contains(&city_contains.to_lowercase()) {
                        matches = false;
                    }
                } else {
                    matches = false;
                }
            }

            if let Some(country_contains) = &expected.country_contains {
                if let Some(country) = &location.country {
                    if !country
                        .to_lowercase()
                        .contains(&country_contains.to_lowercase())
                    {
                        matches = false;
                    }
                } else {
                    matches = false;
                }
            }

            if matches {
                return true;
            }
        }
        false
    }

    fn count_expected_entities(&self, expected: &ExpectedEntities) -> usize {
        let mut count = 0;
        if let Some(persons) = &expected.persons {
            count += persons.len();
        }
        if let Some(locations) = &expected.locations {
            count += locations.len();
        }
        if let Some(events) = &expected.events {
            count += events.len();
        }
        if let Some(products) = &expected.products {
            count += products.len();
        }
        if let Some(organizations) = &expected.organizations {
            count += organizations.len();
        }
        count
    }

    /// Generate a test report and optionally create GitHub issues for failed tests
    pub async fn generate_report(&self, results: Vec<TestResult>, create_issues: bool) -> String {
        let total_tests = results.len();
        let passed_tests = results.iter().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;

        let mut report = format!("# Test Report\n\n");
        report.push_str(&format!("**Total Tests:** {}\n", total_tests));
        report.push_str(&format!("**Passed:** {}\n", passed_tests));
        report.push_str(&format!("**Failed:** {}\n\n", failed_tests));

        for result in &results {
            report.push_str(&format!("## Test: {}\n", result.test_name));
            report.push_str(&format!(
                "**Status:** {}\n",
                if result.passed {
                    "✅ PASSED"
                } else {
                    "❌ FAILED"
                }
            ));
            report.push_str(&format!(
                "**Execution Time:** {}ms\n",
                result.execution_time_ms
            ));
            report.push_str(&format!(
                "**Entities Found:** {}\n",
                result.extracted_entities_count
            ));
            report.push_str(&format!(
                "**Expected Entities:** {}\n",
                result.expected_entities_count
            ));

            if !result.errors.is_empty() {
                report.push_str("\n**Errors:**\n");
                for error in &result.errors {
                    report.push_str(&format!("- {}\n", error));
                }
            }

            if !result.warnings.is_empty() {
                report.push_str("\n**Warnings:**\n");
                for warning in &result.warnings {
                    report.push_str(&format!("- {}\n", warning));
                }
            }

            if !result.analysis_summary.is_empty() {
                report.push_str(&format!(
                    "\n**Analysis Summary:**\n{}\n",
                    result.analysis_summary
                ));
            }

            report.push_str("\n---\n\n");

            // Create GitHub issue for failed tests
            if !result.passed && create_issues {
                if let Err(e) = self.create_github_issue_for_failed_test(result).await {
                    error!(
                        "Failed to create GitHub issue for test '{}': {}",
                        result.test_name, e
                    );
                }
            }
        }

        report
    }

    /// Create a GitHub issue for a failed test
    async fn create_github_issue_for_failed_test(
        &self,
        result: &TestResult,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let title = format!("Test Failure: {}", result.test_name);

        let mut body = format!(
            "# Test Failure Report\n\n\
            **Test Name:** {}\n\
            **Execution Time:** {}ms\n\
            **Entities Found:** {}\n\
            **Expected Entities:** {}\n\n",
            result.test_name,
            result.execution_time_ms,
            result.extracted_entities_count,
            result.expected_entities_count
        );

        if !result.errors.is_empty() {
            body.push_str("## Errors\n");
            for error in &result.errors {
                body.push_str(&format!("- {}\n", error));
            }
            body.push_str("\n");
        }

        if !result.warnings.is_empty() {
            body.push_str("## Warnings\n");
            for warning in &result.warnings {
                body.push_str(&format!("- {}\n", warning));
            }
            body.push_str("\n");
        }

        if !result.analysis_summary.is_empty() {
            body.push_str(&format!(
                "## Analysis Summary\n{}\n\n",
                result.analysis_summary
            ));
        }

        body.push_str("## Suggested Improvements\n");
        body.push_str("Based on the test failure, consider the following improvements:\n\n");

        if result.extracted_entities_count == 0 {
            body.push_str("- Review entity extraction prompts for better accuracy\n");
            body.push_str("- Check if the target URL contains the expected content type\n");
            body.push_str("- Verify that the objective clearly describes what to extract\n");
        } else if result.extracted_entities_count < result.expected_entities_count {
            body.push_str("- Improve URL selection to cover more relevant pages\n");
            body.push_str("- Enhance entity extraction to capture more complete data\n");
            body.push_str(
                "- Review content analysis to ensure all relevant sections are processed\n",
            );
        }

        body.push_str("\n## Development Workflow\n");
        body.push_str("Please follow the development workflow as described in CLAUDE.md:\n");
        body.push_str("- Create a new branch for each task (chore/, feature/, fix/)\n");
        body.push_str("- Add tests to check inputs and outputs\n");
        body.push_str("- Run formatters, linters and tests before committing\n");
        body.push_str("- Create PR when finished\n");

        // Use gh CLI to create the issue
        let output = Command::new("gh")
            .args(&[
                "issue",
                "create",
                "--title",
                &title,
                "--body",
                &body,
                "--label",
                "bug,test-failure",
            ])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to create GitHub issue: {}", error).into());
        }

        let issue_url = String::from_utf8_lossy(&output.stdout);
        info!(
            "Created GitHub issue for failed test '{}': {}",
            result.test_name,
            issue_url.trim()
        );

        Ok(())
    }
}

struct ValidationResult {
    errors: Vec<String>,
    warnings: Vec<String>,
}
