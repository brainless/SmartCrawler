# CLI Options

SmartCrawler provides a simple command-line interface to crawl web pages and extract structured HTML content.

## Basic Usage

```bash
smart-crawler --link <URL> [OPTIONS]
```

## Required Arguments

### `--link <URL>`
- **Description**: URL to crawl (can be specified multiple times)
- **Required**: Yes
- **Multiple values**: Yes

**Examples:**
```bash
# Crawl a single URL
smart-crawler --link "https://example.com"

# Crawl multiple URLs
smart-crawler --link "https://example.com" --link "https://another.com"
```

## Optional Arguments

### `--verbose`
- **Description**: Enable verbose output showing filtered HTML node tree
- **Type**: Flag (no value required)
- **Default**: Disabled

When enabled, this option shows the complete HTML tree structure with duplicate node filtering applied.

**Example:**
```bash
smart-crawler --link "https://example.com" --verbose
```

### `--template`
- **Description**: Enable template detection mode to identify patterns like '{count} comments' in HTML content
- **Type**: Flag (no value required)
- **Default**: Disabled

When enabled, this option:
- Detects variable patterns in text content (e.g., "42 comments" becomes "{count} comments")
- Skips domain-wide duplicate filtering to show template patterns clearly
- Useful for identifying common content patterns across pages

**Example:**
```bash
smart-crawler --link "https://example.com" --template --verbose
```

### `-h, --help`
- **Description**: Print help information
- **Type**: Flag

### `-V, --version`
- **Description**: Print version information
- **Type**: Flag

## Advanced Usage Examples

### Crawl with Template Detection
```bash
smart-crawler --link "https://news-site.com" --template --verbose
```

### Crawl Multiple Domains
```bash
smart-crawler \
  --link "https://example.com" \
  --link "https://another.com" \
  --link "https://third-site.org" \
  --verbose
```

### Debug Mode Output
```bash
# Set log level for detailed debugging
RUST_LOG=debug smart-crawler --link "https://example.com" --verbose
```

## Output Format

SmartCrawler outputs crawling results in a structured format:

```
=== Crawling Results ===
URL: https://example.com
Title: Example Domain
Domain: example.com
---
```

With `--verbose` enabled, it also shows the HTML tree structure with duplicate filtering applied.

With `--template` enabled, it shows template patterns instead of actual values and skips duplicate filtering.

## Exit Codes

- `0`: Success
- `1`: Error (invalid arguments, connection failure, etc.)

## Environment Variables

- `RUST_LOG`: Set logging level (`debug`, `info`, `warn`, `error`)

## Notes

- URLs are automatically validated and deduplicated
- SmartCrawler requires a WebDriver server running on port 4444
- See the [Getting Started guides](README.md#getting-started) for WebDriver setup instructions