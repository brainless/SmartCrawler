# SmartCrawler Testing Guide

This guide explains how to write and run automated tests for SmartCrawler to verify that the crawler correctly extracts expected data from web pages.

## Overview

SmartCrawler includes an automated testing system that allows you to:
- Define test cases with expected data extraction results
- Run tests against real websites 
- Validate that the crawler finds the expected entities
- Automatically create GitHub issues when tests fail with detected problems

## Test File Format

Tests are defined in JSON files with the following structure:

```json
{
  "tests": [
    {
      "name": "Test Name",
      "description": "Description of what this test validates",
      "url": "https://example.com/page-to-test",
      "objective": "Find contact information for team members",
      "expected_entities": {
        "persons": [
          {
            "name_contains": "john",
            "title_contains": "ceo"
          }
        ],
        "locations": [
          {
            "city_contains": "san francisco",
            "country_contains": "usa"
          }
        ],
        "minimum_entities_count": 2
      },
      "timeout_seconds": 30
    }
  ]
}
```

## Test Structure Fields

### Test Case Fields

- **name** (required): A descriptive name for the test
- **description** (required): What the test is validating
- **url** (required): The URL to test data extraction on
- **objective** (required): The crawling objective (what to look for)
- **expected_entities** (required): Definition of expected entities
- **timeout_seconds** (optional): Test timeout in seconds (default: 30)

### Expected Entities

The `expected_entities` object can contain:

- **persons**: Array of person validation criteria
- **locations**: Array of location validation criteria  
- **events**: Array of event validation criteria
- **products**: Array of product validation criteria
- **organizations**: Array of organization validation criteria
- **minimum_entities_count**: Minimum total entities expected

### Person Validation Criteria

```json
{
  "name_contains": "partial name to match",
  "title_contains": "partial title to match",
  "company_contains": "partial company to match"
}
```

### Location Validation Criteria

```json
{
  "name_contains": "partial location name",
  "city_contains": "partial city name", 
  "country_contains": "partial country name"
}
```

### Event Validation Criteria

```json
{
  "title_contains": "partial event title",
  "location_contains": "partial event location"
}
```

### Product Validation Criteria

```json
{
  "name_contains": "partial product name",
  "brand_contains": "partial brand name"
}
```

### Organization Validation Criteria

```json
{
  "name_contains": "partial organization name",
  "industry_contains": "partial industry"
}
```

## Running Tests

### Basic Test Execution

```bash
# Run tests from a JSON file
cargo run -- --test tests/my-tests.json

# Run tests with verbose logging
cargo run -- --test tests/my-tests.json --verbose
```

### Create GitHub Issues for Failed Tests

```bash
# Automatically create GitHub issues for failed tests
cargo run -- --test tests/my-tests.json --create-issues
```

This requires the `gh` CLI tool to be installed and authenticated.

## Example Test File

Here's a complete example test file (`example-test.json`):

```json
{
  "tests": [
    {
      "name": "Anthropic About Page Team Test",
      "description": "Test that we can extract team member information from Anthropic's about page",
      "url": "https://www.anthropic.com/company",
      "objective": "Find information about company leadership and team members",
      "expected_entities": {
        "persons": [
          {
            "name_contains": "dario",
            "title_contains": "ceo"
          }
        ],
        "organizations": [
          {
            "name_contains": "anthropic"
          }
        ],
        "minimum_entities_count": 1
      },
      "timeout_seconds": 45
    },
    {
      "name": "University Contact Page Test", 
      "description": "Test extraction of contact information from a university page",
      "url": "https://www.stanford.edu/about/",
      "objective": "Find contact information and location details",
      "expected_entities": {
        "locations": [
          {
            "city_contains": "stanford",
            "state_contains": "california"
          }
        ],
        "organizations": [
          {
            "name_contains": "stanford"
          }
        ],
        "minimum_entities_count": 2
      }
    }
  ]
}
```

## Test Validation Logic

The test system validates extracted entities using partial string matching:

- **Case-insensitive matching**: All text comparisons are case-insensitive
- **Partial matching**: Uses "contains" logic rather than exact matching
- **Optional fields**: Only checks fields that are specified in expected entities
- **Confidence warnings**: Warns if extraction confidence is below 50%

## Failed Test Reports

When tests fail, the system generates detailed reports showing:

- Which expected entities were not found
- Total entities extracted vs expected
- Extraction confidence scores
- Analysis summary from the crawler
- Suggestions for algorithm improvements

## Automatic GitHub Issue Creation

When `--create-issues` is enabled, failed tests automatically create GitHub issues with:

- Detailed failure analysis
- Suggested improvements to the crawler algorithms
- Links to the development workflow in CLAUDE.md
- Bug and test-failure labels

## Best Practices

1. **Specific Objectives**: Write clear, specific objectives that describe exactly what data to extract

2. **Partial Matching**: Use partial strings in expected entities to account for variations in extracted text

3. **Multiple Criteria**: Include multiple validation criteria per entity type for more robust testing

4. **Minimum Counts**: Use `minimum_entities_count` to ensure the crawler finds sufficient data

5. **Regular Testing**: Run tests regularly to catch regressions in crawler performance

6. **Real Websites**: Test against real websites that contain the types of data your crawler needs to handle

7. **Edge Cases**: Include tests for edge cases like pages with minimal content, complex layouts, or unusual data structures

## Troubleshooting

### Test Failures

If tests fail, check:
- The target URL is accessible and contains the expected content
- The objective clearly describes what to look for
- Expected entity criteria match the actual content structure
- The crawler has sufficient context to make good extractions

### GitHub Issue Creation Failures

If automatic issue creation fails:
- Ensure `gh` CLI is installed and authenticated
- Check repository permissions for issue creation
- Verify you're in the correct repository directory

### Performance Issues

For slow tests:
- Increase timeout values for complex pages
- Consider testing on simpler, faster-loading pages
- Check network connectivity and target site responsiveness