# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Core Commands

### Build and Run
```bash
# Build the project
cargo build --release

# Run with basic command
cargo run -- --objective "Find pricing information" --domains "example.com" --max-urls 5

# Run with all options
cargo run -- -o "Your objective" -d "domain1.com,domain2.com" -m 10 --delay 1000 -O results.json -v
```

### Development Commands  
```bash
# Check compilation without building
cargo check

# Run with debug logging
RUST_LOG=debug cargo run -- [args]

# Format code
cargo fmt

# Run clippy for linting
cargo clippy
```

## Architecture Overview

SmartCrawler is a Rust-based intelligent web crawler that uses LLMs (primarily Claude) to make smart decisions about which URLs to crawl based on crawling objectives.

### Core Components

**Main Flow**: `main.rs` → `SmartCrawler` → `LLM trait` → Results
- Entry point parses CLI args and creates `CrawlerConfig`
- Instantiates `ClaudeClient` and wraps in `Arc` for `SmartCrawler`
- Calls `crawl_all_domains()` which processes each domain sequentially

**SmartCrawler** (`src/crawler.rs`):
- Central orchestrator that manages the crawling workflow
- Uses Arc<Mutex<Vec<String>>> for `domains` to allow dynamic domain addition during crawling
- Tracks scraped URLs per domain to avoid duplicates
- Handles both sitemap-based and fallback crawling strategies

**LLM Abstraction** (`src/llm.rs`):
- Defines `LLM` trait for swappable AI providers
- Currently implemented only by `ClaudeClient`
- Key methods: `select_urls()`, `analyze_content()`, `send_message()`

**Crawling Strategy**:
1. **Sitemap Discovery**: Uses `SitemapParser` to find XML sitemaps
2. **Fallback Strategy**: If no sitemap, scrapes homepage for links
3. **LLM URL Selection**: AI selects most relevant URLs based on objective
4. **Content Scraping**: Uses headless browser via `Browser` wrapper
5. **Content Analysis**: LLM analyzes scraped content against objective

### Key Design Patterns

**Error Handling**: Custom `CrawlerError` enum that wraps underlying errors from sitemap, LLM, browser, and I/O operations.

**Concurrency**: 
- `Arc<Mutex<>>` for shared state (domains, scraped URLs)
- Async/await throughout for I/O operations
- Single-threaded execution per domain (sequential processing)

**Configuration**: `CrawlerConfig` holds all runtime parameters, with Arc<Mutex<Vec<String>>> for domains to enable dynamic addition.

## Environment Setup

Requires `ANTHROPIC_API_KEY` environment variable. The application validates this on startup.

Optional: Create `.env` file with:
```
ANTHROPIC_API_KEY=your_key_here
```

## Development Notes

- The project uses a trait-based LLM abstraction but currently only supports Claude
- Browser automation uses `fantoccini` crate for headless browsing
- Sitemap parsing handles both regular sitemaps and sitemap indexes
- URL deduplication is implemented per-domain to avoid re-crawling
- The crawler can dynamically add domains during execution (though this feature isn't exposed via CLI)
- Rate limiting is implemented via configurable delays between requests
- Results are saved in structured JSON format for further analysis

## Git Workflow
- Create a new branch for each task. Branch names should start with chore/ or feature/ or fix/ etc.

## Task Management Workflow
- When starting a new task please save the user's request and task plan to VIBE.md