# Real-World Integration Tests

This directory contains integration tests that crawl real websites to verify SmartCrawler's complete functionality in real-world scenarios, including domain-level duplicate filtering.

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
# Run all real-world tests (they run serially to avoid WebDriver conflicts)
cargo test real_world -- --ignored

# Run a specific test
cargo test test_hacker_news_submissions -- --ignored
cargo test test_mykin_ai_team_member -- --ignored

# Test WebDriver connection
cargo test test_webdriver_connection -- --ignored
```

### Serial Execution

**Important**: Real-world tests are configured to run serially (one at a time) using the `serial_test` crate. This prevents WebDriver session conflicts that occur when multiple tests try to use the same WebDriver port (4444) simultaneously.

The `#[serial]` attribute ensures that:
- Tests won't interfere with each other's WebDriver sessions
- Each test gets exclusive access to the WebDriver instance
- Tests are more reliable and predictable

## Test Descriptions

### `test_hacker_news_submissions`
- **URL**: https://news.ycombinator.com/
- **Purpose**: Verifies the complete SmartCrawler pipeline including domain-level duplicate filtering
- **Pipeline**: Full 3-phase crawling with link discovery, root URL prioritization, and duplicate analysis
- **Path**: `html body center table tbody tr td table tbody tr.athing.submission td.title`
- **Expected**: Exactly 30 elements matching this path (including filtered duplicates)
- **Features Tested**: Link discovery, domain duplicate detection, filtered content identification

### `test_mykin_ai_team_member`
- **URL**: https://mykin.ai/company
- **Purpose**: Verifies the complete SmartCrawler pipeline on a modern web framework
- **Pipeline**: Full 3-phase crawling with link discovery, root URL prioritization, and duplicate analysis
- **Path**: `html.w-mod-js.w-mod-ix body div.page-wrapper main.main-wrapper section.section_team div.padding-global div.container-medium div.team_collection.is-desktop.w-dyn-list div.team_collection-list.w-dyn-items div.w-dyn-item a.team_card.w-inline-block div.team_content h4`
- **Expected**: At least one element containing "Kasper Juul" (excluding filtered duplicates)
- **Features Tested**: Complex CSS class matching, content filtering, modern web app crawling

## Manual Verification

If a test fails, you should manually verify the webpage structure:

1. Visit the URL in your browser
2. Inspect the HTML structure using browser developer tools
3. Check if the CSS path still matches the expected elements
4. Update the test path if the website structure has changed

## SmartCrawler Pipeline Features Tested

The real-world tests exercise the complete SmartCrawler functionality:

### Phase 1: Preparation Stage
- **Link Discovery**: Extracts same-domain links from initial page
- **Root URL Prioritization**: Automatically adds domain root URLs
- **URL Collection**: Gathers up to 3 URLs per domain for analysis

### Phase 2: Processing Stage  
- **Sequential Processing**: Processes URLs in priority order (user → root → discovered)
- **Content Extraction**: Retrieves HTML source and parses into structured tree
- **Status Tracking**: Monitors fetch success/failure for each URL

### Phase 3: Analysis Stage
- **Domain Duplicate Detection**: Identifies common elements across domain pages
- **Content Filtering**: Marks duplicate patterns as `[FILTERED DUPLICATE]`
- **Result Preparation**: Provides clean, filtered HTML trees for analysis

## Notes

- These tests depend on external websites and may fail if:
  - The websites are down or inaccessible
  - The website structure has changed
  - Network connectivity issues
  - WebDriver is not running or accessible
- Element IDs are ignored in path matching to make tests more robust against dynamic content
- Tests include verbose output to help with debugging when they fail
- **Serial execution**: Tests run one at a time to prevent WebDriver session conflicts - this is automatic and requires no special flags
- **Full pipeline testing**: Each test exercises the complete 3-phase SmartCrawler workflow
- **Duplicate filtering**: Tests verify that domain-level filtering correctly identifies and marks duplicate content