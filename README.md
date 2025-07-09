# SmartCrawler

A web crawler that uses WebDriver to extract and parse HTML content from web pages with intelligent duplicate detection and template pattern recognition.

## ✨ Features

- **🌐 Multi-URL Crawling**: Crawl multiple URLs in a single session
- **🔍 Intelligent Duplicate Detection**: Automatically identifies and filters duplicate content patterns across domains
- **📋 Template Pattern Recognition**: Detects variable patterns in content (e.g., "42 comments" → "{count} comments")
- **🌳 Structured HTML Tree**: Provides filtered HTML tree view with duplicate marking
- **⚡ WebDriver Integration**: Uses WebDriver for dynamic content handling
- **📊 Verbose Output**: Detailed HTML tree analysis with filtering information

## 🚀 Quick Start

1. **Install SmartCrawler** - Download from [releases](https://github.com/pixlie/SmartCrawler/releases) or build from source
2. **Set up WebDriver** - Install Firefox/Chrome and corresponding WebDriver
3. **Start crawling** - Run SmartCrawler with your target URLs

```bash
# Basic usage
smart-crawler --link "https://example.com"

# Multiple URLs with verbose output
smart-crawler --link "https://example.com" --link "https://another.com" --verbose

# Template detection mode
smart-crawler --link "https://example.com" --template --verbose
```

## 📖 Documentation

### Getting Started

Choose your operating system for detailed setup instructions:

- **[Windows Setup](docs/getting-started-windows.md)** - Complete Windows installation guide
- **[macOS Setup](docs/getting-started-macos.md)** - macOS installation and setup
- **[Linux Setup](docs/getting-started-linux.md)** - Linux installation for various distributions

### Usage

- **[CLI Options](docs/cli-options.md)** - Complete command-line reference and examples

### Development

- **[Development Guide](docs/development.md)** - Setup, building, testing, and contributing instructions

## 🔧 System Requirements

- **Operating System**: Windows 10+, macOS 10.15+, or Linux
- **Browser**: Firefox (recommended) or Chrome
- **WebDriver**: GeckoDriver (Firefox) or ChromeDriver (Chrome)
- **Memory**: 512MB RAM minimum, 1GB recommended

## 📋 Quick Reference

### Basic Commands

```bash
# Crawl a single URL
smart-crawler --link "https://example.com"

# Crawl with detailed output
smart-crawler --link "https://example.com" --verbose

# Template detection mode
smart-crawler --link "https://example.com" --template --verbose

# Multiple URLs
smart-crawler --link "https://site1.com" --link "https://site2.com"
```

### WebDriver Setup

```bash
# Start Firefox WebDriver
geckodriver --port 4444

# Start Chrome WebDriver
chromedriver --port=4444
```

## 🛠️ Development

For developers interested in contributing to SmartCrawler or building from source:

- **[Development Guide](docs/development.md)** - Complete setup, building, testing, and contributing instructions

## 📄 License

This project is licensed under the GPL-3.0 license - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- [GitHub Repository](https://github.com/pixlie/SmartCrawler)
- [Issue Tracker](https://github.com/pixlie/SmartCrawler/issues)
- [Releases](https://github.com/pixlie/SmartCrawler/releases)
- [Documentation](docs/)

## 🆘 Support

If you encounter issues:

1. Check the [getting started guides](docs/) for your operating system
2. Review the [CLI options documentation](docs/cli-options.md)
3. Search existing [GitHub issues](https://github.com/pixlie/SmartCrawler/issues)
4. Create a new issue with detailed error information

---

**Note**: SmartCrawler is designed for ethical web scraping and research purposes. Always respect websites' robots.txt files and terms of service.
