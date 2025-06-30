# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Workflow
- Create a new branch for each task. Branch names should start with chore/ or feature/ or fix/ etc.
- When starting a new task please save the user's request and task plan to VIBE.md
- When finished please commit and push to the new branch
- Please add tests to check inputs and outputs that are user facing
- Please mention GitHub issue if provided

## Core Commands

### Build and Run
```bash
# Build the project
cargo build --release

# Run with basic command
cargo run -- --objective "Find pricing information" --domains "example.com" --max-urls 5

# Run with all options including keyword filtering
cargo run -- -o "Your objective" -d "domain1.com,domain2.com" -m 10 --delay 1000 -O results.json -v --candidate-multiplier 5
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
- Central orchestrator with two-stage URL selection (keyword ranking + LLM selection)
- Tracks scraped URLs per domain to avoid duplicates
- Handles sitemap-based and fallback crawling strategies

**LLM Abstraction** (`src/llm.rs`):
- Defines `LLM` trait for swappable AI providers
- Key methods: `generate_keywords()`, `select_urls()`, `analyze_content()`, `extract_entities()`

**URL Selection Strategy**:
1. **Sitemap Discovery**: Find XML sitemaps or scrape homepage for links
2. **Keyword Generation**: LLM generates relevant keywords from objective
3. **URL Ranking**: Score URLs based on keyword relevance in paths/queries
4. **LLM Selection**: AI selects best URLs from top-ranked candidates
5. **Content Analysis**: Extract structured entities matching TypeScript schemas

## Environment Setup

Requires `ANTHROPIC_API_KEY` environment variable. Create `.env` file:
```
ANTHROPIC_API_KEY=your_key_here
```
