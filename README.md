# SmartCrawler

A smart web crawler that uses AI to intelligently select and analyze web pages based on your specific objectives. SmartCrawler automatically discovers sitemaps, selects the most relevant URLs using Claude AI, and provides detailed analysis of the content it finds.

## 📋 Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [WebDriver Setup](#webdriver-setup)
- [Usage](#usage)
- [Examples](#examples)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)
- [License](#license)

## ✨ Features

- **🤖 AI-Powered URL Selection**: Uses Claude AI to intelligently select relevant URLs from sitemaps
- **🗺️ Automatic Sitemap Discovery**: Finds and parses XML sitemaps across multiple domains
- **📄 Smart Content Analysis**: AI-powered analysis of scraped content for objective-specific insights
- **🌐 Multi-Domain Support**: Crawl multiple websites in a single session
- **⚡ Dynamic Content Loading**: Scrolls through pages to capture JavaScript-rendered content
- **📊 Structured Output**: Results saved in JSON format for further analysis
- **🛡️ Respectful Crawling**: Built-in rate limiting and respectful crawling practices

## 🚀 Quick Start

1. **Download SmartCrawler** from the [releases page](https://github.com/brainless/SmartCrawler/releases)
2. **Set up WebDriver** (Firefox or Chrome)
3. **Get Claude API key** from [Anthropic](https://console.anthropic.com/)
4. **Run SmartCrawler** with your objective

## 📦 Installation

### Option 1: Download Pre-built Binary (Recommended)

**Windows:**
1. Download `smart-crawler-windows-x64.zip` from [releases](https://github.com/brainless/SmartCrawler/releases)
2. Extract the ZIP file to a folder (e.g., `C:\SmartCrawler\`)
3. Add the folder to your PATH or run from the folder directly

**macOS:**
1. Download `smart-crawler-macos-x64.tar.gz` (Intel) or `smart-crawler-macos-arm64.tar.gz` (Apple Silicon)
2. Extract: `tar -xzf smart-crawler-macos-*.tar.gz`
3. Move to applications: `sudo mv smart-crawler /usr/local/bin/`
4. Make executable: `chmod +x /usr/local/bin/smart-crawler`

**Linux:**
1. Download `smart-crawler-linux-x64.tar.gz`
2. Extract: `tar -xzf smart-crawler-linux-x64.tar.gz`
3. Move to bin: `sudo mv smart-crawler /usr/local/bin/`
4. Make executable: `chmod +x /usr/local/bin/smart-crawler`

### Option 2: Package Installers

**Windows MSI Installer:**
1. Download `smart-crawler-[version].msi` from releases
2. Double-click to install
3. SmartCrawler will be available in your PATH

**Linux DEB Package (Ubuntu/Debian):**
```bash
wget https://github.com/brainless/SmartCrawler/releases/latest/download/smart-crawler-[version].deb
sudo dpkg -i smart-crawler-[version].deb
```

**Linux RPM Package (RHEL/CentOS/Fedora):**
```bash
wget https://github.com/brainless/SmartCrawler/releases/latest/download/smart-crawler-[version].rpm
sudo rpm -i smart-crawler-[version].rpm
```

**macOS DMG:**
1. Download `smart-crawler-[version].dmg` from releases
2. Open the DMG and copy `smart-crawler` to `/usr/local/bin/`

### Option 3: Build from Source

If you have Rust installed:
```bash
git clone https://github.com/brainless/SmartCrawler.git
cd SmartCrawler
cargo build --release
# Binary will be in target/release/smart-crawler
```

## 🌐 WebDriver Setup

SmartCrawler requires a WebDriver to control a browser for scraping. Choose one:

### Firefox (GeckoDriver) - Recommended

**Windows:**
1. Download `geckodriver.exe` from [Mozilla releases](https://github.com/mozilla/geckodriver/releases)
2. Place in a folder in your PATH or the same folder as SmartCrawler
3. Install Firefox browser if not already installed

**macOS:**
```bash
# Using Homebrew (recommended)
brew install geckodriver

# Or download manually from releases page
```

**Linux:**
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install firefox-esr
wget https://github.com/mozilla/geckodriver/releases/latest/download/geckodriver-v0.33.0-linux64.tar.gz
tar -xzf geckodriver-v0.33.0-linux64.tar.gz
sudo mv geckodriver /usr/local/bin/

# RHEL/CentOS/Fedora
sudo dnf install firefox
# Then download geckodriver as above
```

### Chrome (ChromeDriver) - Alternative

**Windows:**
1. Download ChromeDriver from [Chrome for Testing](https://googlechromelabs.github.io/chrome-for-testing/)
2. Place `chromedriver.exe` in your PATH or SmartCrawler folder
3. Install Chrome browser if not already installed

**macOS:**
```bash
# Using Homebrew
brew install chromedriver

# Or download manually
```

**Linux:**
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install google-chrome-stable
wget https://chromedriver.storage.googleapis.com/[VERSION]/chromedriver_linux64.zip
unzip chromedriver_linux64.zip
sudo mv chromedriver /usr/local/bin/

# Check Chrome version: google-chrome --version
# Download matching ChromeDriver version
```

## 🎯 Usage

### 1. Set up your Claude API key

**Windows (Command Prompt):**
```cmd
set ANTHROPIC_API_KEY=your-api-key-here
```

**Windows (PowerShell):**
```powershell
$env:ANTHROPIC_API_KEY="your-api-key-here"
```

**macOS/Linux:**
```bash
export ANTHROPIC_API_KEY="your-api-key-here"
```

### 2. Start WebDriver

**For Firefox (GeckoDriver):**
```bash
# Windows
geckodriver.exe --port 4444

# macOS/Linux  
geckodriver --port 4444
```

**For Chrome (ChromeDriver):**
```bash
# Windows
chromedriver.exe --port=4444

# macOS/Linux
chromedriver --port=4444
```

### 3. Run SmartCrawler

**Basic usage:**
```bash
smart-crawler --objective "Find pricing information" --domains "example.com" --max-urls 5
```

**With output file:**
```bash
smart-crawler -o "Find contact information" -d "company.com,business.org" -m 10 -O results.json
```

**Command-line options:**
```
-o, --objective <OBJECTIVE>    What information to look for [REQUIRED]
-d, --domains <DOMAINS>        Comma-separated domains to crawl [REQUIRED]
-m, --max-urls <NUMBER>        Maximum URLs per domain [default: 10]
    --delay <MILLISECONDS>     Delay between requests [default: 1000]
-O, --output <FILE>           Save results to JSON file
-v, --verbose                 Enable detailed logging
```

## 📝 Examples

### E-commerce Price Research
```bash
smart-crawler \
  --objective "Find product pricing, discounts, and shipping costs" \
  --domains "shop1.com,shop2.com,competitor.com" \
  --max-urls 15 \
  --output pricing-research.json
```

### Company Information Gathering
```bash
smart-crawler \
  -o "Find company contact information, team members, and office locations" \
  -d "company.com" \
  -m 8 \
  --delay 2000 \
  -v
```

### Technical Documentation Search
```bash
smart-crawler \
  --objective "Find API documentation, integration guides, and developer resources" \
  --domains "docs.example.com,api.service.com" \
  --max-urls 20 \
  --output api-docs.json
```

### News and Content Analysis
```bash
smart-crawler \
  -o "Find recent news articles about artificial intelligence and machine learning" \
  -d "news-site.com,tech-blog.com" \
  -m 12 \
  --delay 1500
```

## ⚙️ Configuration

### Environment Variables

- `ANTHROPIC_API_KEY`: Your Claude API key (required)
- `RUST_LOG`: Set logging level (`debug`, `info`, `warn`, `error`)

### Best Practices

- **Specific objectives**: Use clear, specific objectives for better URL selection
- **Conservative limits**: Start with lower `--max-urls` values (5-10) to test
- **Respectful delays**: Use delays of 1000ms or more to avoid overwhelming servers
- **Check robots.txt**: Review target sites' crawling policies
- **Monitor API usage**: Claude API has usage limits and costs

## 🔧 Troubleshooting

### Common Issues

**"WebDriver connection failed"**
- Ensure WebDriver (geckodriver/chromedriver) is running on port 4444
- Check that the browser is installed
- Try restarting the WebDriver

**"ANTHROPIC_API_KEY not found"**
- Set the environment variable in your terminal session
- Verify the API key is correct and has sufficient credits

**"Permission denied" (macOS/Linux)**
- Make the binary executable: `chmod +x smart-crawler`
- For system-wide installation, use `sudo` when moving to `/usr/local/bin/`

**"No URLs selected by LLM"**
- Try a more specific objective
- Increase `--max-urls` limit
- Use `--verbose` to see what URLs were found

**Rate limiting/blocked requests**
- Increase `--delay` between requests
- Some sites block crawlers; respect robots.txt
- Consider using fewer concurrent requests

### Getting Help

- Check the [Issues page](https://github.com/brainless/SmartCrawler/issues) for known problems
- Create a new issue with detailed error messages and system information
- Include your command and any error output when reporting problems

## 📄 License

GPL-3.0 License - see [LICENSE](LICENSE) file for details.

---

**Note**: SmartCrawler is designed for ethical web scraping and research purposes. Always respect websites' terms of service and robots.txt files. Be mindful of rate limits and server resources when crawling.