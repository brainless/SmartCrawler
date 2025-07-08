# Real-World Integration Tests

This directory contains integration tests that crawl real websites to verify SmartCrawler's functionality in real-world scenarios.

## Prerequisites

Before running these tests, you need to have a WebDriver server running:

### Option 1: GeckoDriver (Firefox) - Recommended
```bash
# Install geckodriver and start it
geckodriver
```

### Option 2: ChromeDriver
```bash
# Download ChromeDriver and start it on port 4444
chromedriver --port=4444
```

### Option 3: Docker
```bash
# Run Chrome in headless mode with WebDriver
docker run -d -p 4444:4444 selenium/standalone-chrome:latest
```

## Running the Tests

The real-world tests are ignored by default to prevent them from running during normal development. To run them explicitly:

```bash
# Run all real-world tests
cargo test real_world -- --ignored

# Run a specific test
cargo test test_hacker_news_submissions -- --ignored
cargo test test_mykin_ai_team_member -- --ignored

# Test WebDriver connection
cargo test test_webdriver_connection -- --ignored
```

## Test Descriptions

### `test_hacker_news_submissions`
- **URL**: https://news.ycombinator.com/
- **Purpose**: Verifies that the HTML parser can correctly identify 30 submission elements
- **Path**: `html body center table tbody tr td table tbody tr.athing.submission td.title`
- **Expected**: Exactly 30 elements matching this path

### `test_mykin_ai_team_member`
- **URL**: https://mykin.ai/company
- **Purpose**: Verifies that the HTML parser can find a specific team member
- **Path**: `html.w-mod-js.w-mod-ix body div.page-wrapper main.main-wrapper section.section_team div.padding-global div.container-medium div.team_collection.is-desktop.w-dyn-list div.team_collection-list.w-dyn-items div.w-dyn-item a.team_card.w-inline-block div.team_content h4`
- **Expected**: At least one element containing "Kasper Juul"

## Manual Verification

If a test fails, you should manually verify the webpage structure:

1. Visit the URL in your browser
2. Inspect the HTML structure using browser developer tools
3. Check if the CSS path still matches the expected elements
4. Update the test path if the website structure has changed

## Notes

- These tests depend on external websites and may fail if:
  - The websites are down or inaccessible
  - The website structure has changed
  - Network connectivity issues
  - WebDriver is not running or accessible
- Element IDs are ignored in path matching to make tests more robust against dynamic content
- Tests include verbose output to help with debugging when they fail