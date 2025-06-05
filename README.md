# Smart Crawler

An intelligent web crawler built in Rust that uses Claude AI to select the most relevant URLs from website sitemaps based on crawling objectives.

## Created with Claude Code
### Initial prompt

> Please create a crawler and scraper in Rust. You can use any crates you want. The crawler will be given an objective and list of domains via command prompt. Objective may ask for existence of some information or list of extracted data or similar. The crawler should find the sitemap for any domain and ask Claude which URLs make most sense to crawl for the given objective. The crawler should be conservative in   crawling and try to use Claude to reach results fast.

The first version did not handle when sitemap file is not found, so I tried:

> If a sitemap file is not found for a domain, can you generate a SiteMap from the links hierarchy found from the homepage or other pages? Then refer to this SiteMap as you continue crawling and base your decisions on it.

This did not work well, the way Claude wrote the code was to first crawl a bunch of URLs to generate a possible sitemap. It does not work, not sure why but it would also defeat the goal of a conservative crawl.

Here are the next prompts I tried:

> Regarding CrawlerConfig.domains, will it work if domains get added while the domains are being crawled in the loop in SmartCrawler::crawl_all_domains?

> Can we update CrawlerConfig.domains so that domains can be added while existing domains are being crawled in the loop in SmartCrawler::crawl_all_domains?

> Please modify ClaudeClient::select_urls to accept urls of type SitemapUrl or just regular URLs.

> In SmartCrawler keep track of all the URLs that are being scraped for each domain in the function crawl_domain. Only unique URLs allowed.

> In SmartCrawler::crawl_domain if sitemap_urls.is_empty then we should not return immediately. Instead we should use the root URL for the domain and start with it.

> In ClaudeClient::select_urls please update prompt to only return URLs that in the existing list of urls argument. Also please check the response from Claude and ignore return URLs that are not in the existing list of urls.

> In ClaudeClient::select_urls please add an info log of the urls argument.

> In SmartCrawler::crawl_domain if sitemap_urls.is_empty then we should take the root url of the domain and scrape that URL to get all the URLs in it.

## Features

- **Sitemap Discovery**: Automatically finds and parses XML sitemaps for any domain
- **AI-Powered URL Selection**: Uses Claude to intelligently select relevant URLs based on objectives
- **Conservative Crawling**: Implements rate limiting and respectful crawling practices
- **Content Analysis**: Leverages Claude to analyze scraped content for objective-specific insights
- **Multi-Domain Support**: Crawl multiple domains in a single session
- **Structured Output**: Results saved in JSON format for further analysis

## Prerequisites

- Rust 1.70+ installed
- Anthropic API key (Claude)

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd smart-crawler
```

2. Build the project:
```bash
cargo build --release
```

## Usage

### Set up your Anthropic API key:
```bash
export ANTHROPIC_API_KEY="your-api-key-here"
```

### Run the crawler:
```bash
cargo run -- --objective "Find pricing information" --domains "example.com,another-site.com" --max-urls 5
```

### Command-line options:
```
-o, --objective <OBJECTIVE>    The crawling objective - what information to look for [required]
-d, --domains <DOMAINS>        Comma-separated list of domains to crawl [required]
-m, --max-urls <NUMBER>        Maximum URLs to crawl per domain [default: 10]
    --delay <MILLISECONDS>     Delay between requests in milliseconds [default: 1000]
-O, --output <FILE>           Output file for results (JSON format)
-v, --verbose                 Enable verbose logging
```

### Example Usage:

```bash
# Look for pricing information on multiple e-commerce sites
cargo run -- -o "Find product pricing and discount information" -d "shop1.com,shop2.com" -m 8 --output results.json

# Research company information
cargo run -- -o "Find company contact information and team details" -d "company.com" -m 5 -v

# Technical documentation search
cargo run -- -o "Find API documentation and integration guides" -d "api-docs.com,developer-site.com" -m 15
```

## How it Works

1. **Sitemap Discovery**: For each domain, the crawler:
   - Checks common sitemap locations (`/sitemap.xml`, `/sitemap_index.xml`, etc.)
   - Parses `robots.txt` for sitemap references
   - Handles both regular sitemaps and sitemap indexes

2. **AI URL Selection**: Claude AI analyzes all discovered URLs and selects the most relevant ones based on:
   - URL structure and naming patterns
   - Likely content types
   - Relevance to the specified objective

3. **Content Scraping**: The crawler:
   - Respects rate limits with configurable delays
   - Extracts clean text content, titles, and metadata
   - Handles various HTML structures intelligently

4. **AI Content Analysis**: Claude analyzes each scraped page to:
   - Determine relevance to the objective
   - Extract key information and insights
   - Provide structured analysis results

5. **Results Compilation**: Generates comprehensive reports including:
   - Per-domain summaries
   - Overall findings across all domains
   - Structured JSON output for further processing

## Output Format

Results are saved in JSON format containing:
- Crawling objective and target domains
- Selected URLs for each domain
- Scraped content with metadata
- AI analysis for each page
- Domain-specific and overall summaries

## Best Practices

- Use specific, clear objectives for better URL selection
- Start with conservative max-url limits to test effectiveness
- Use appropriate delays (1000ms+) to be respectful to target sites
- Review robots.txt and terms of service for target domains
- Monitor API usage when processing large numbers of URLs

## Limitations

- Requires valid Anthropic API key with sufficient credits
- Subject to rate limits of both the Claude API and target websites
- JavaScript-rendered content may not be fully captured
- Some sites may block automated crawling

## License

MIT License
