# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Workflow
- Create a new branch for each task. Branch names should start with chore/ or feature/ or fix/ etc.
- Please add tests for any new features added
- Please run formatters, linters and tests before committing changes
- When finished please commit and push to the new branch
- Please mention GitHub issue if provided

## Core Commands

### Build and Run
```bash
# Build the project
cargo build --release

# Run with basic command
cargo run -- --link "https://example.com"

# Run with multiple links
cargo run -- --link "https://example.com" --link "https://another.com"
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

# Run tests
cargo test
```

## Architecture Overview

SmartCrawler is a Rust-based web crawler that uses WebDriver to extract and parse HTML content from web pages.

### Core Components

**Main Flow**: `main.rs` → CLI parsing → URL processing → Browser automation → HTML parsing → Results display

**CLI Interface**: 
- Accepts `--link` arguments for URLs to crawl
- Handles duplicate URL detection
- Validates and processes input arguments

**WebDriver Integration**:
- Uses WebDriver to open URLs in local browser
- Extracts HTML source via JavaScript execution
- Handles browser automation and error cases

**HTML Processing**:
- Parses HTML into structured node tree
- Applies filtering rules for relevant content
- Extracts page titles and structured data

**Storage System**:
- Maintains URL storage per domain
- Tracks fetch status and HTML data per URL
- Ensures unique URLs only

## Environment Setup

### WebDriver Setup

SmartCrawler requires a WebDriver server to be running for browser automation. Follow these steps:

#### Option 1: GeckoDriver (Firefox) - Recommended
1. Download GeckoDriver from https://github.com/mozilla/geckodriver/releases
2. Start GeckoDriver (uses port 4444 by default):
   ```bash
   geckodriver
   ```

#### Option 2: ChromeDriver
1. Download ChromeDriver from https://chromedriver.chromium.org/
2. Extract and place it in your PATH or a local directory
3. Start ChromeDriver on port 4444:
   ```bash
   chromedriver --port=4444
   ```

#### Option 3: Using Docker
```bash
# Run Chrome in headless mode with WebDriver
docker run -d -p 4444:4444 selenium/standalone-chrome:latest
```
   
#### Quick Start (if you have geckodriver installed)
```bash
# Start geckodriver in background (uses port 4444 by default)
geckodriver &

# Run SmartCrawler
cargo run -- --link "https://example.com"

# Stop geckodriver when done
pkill geckodriver
```

### Verification
Test that WebDriver is running:
```bash
curl http://localhost:4444/status
```

You should see a JSON response indicating the WebDriver server is ready.

## Testing

Tests cover:
- Text trimming and cleaning utilities
- CLI argument parsing and validation
- WebDriver browser opening and error handling
- HTML parsing rules and node tree creation
- URL storage and deduplication