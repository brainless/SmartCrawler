# Getting Started on macOS

This guide will help you set up SmartCrawler on macOS systems.

## Prerequisites

- macOS 10.15 (Catalina) or later
- Administrator access for installation
- Internet connection for downloads

## Step 1: Install SmartCrawler

### Option A: Download Pre-built Binary (Recommended)

1. Go to the [SmartCrawler releases page](https://github.com/pixlie/SmartCrawler/releases)
2. Download the appropriate binary for your Mac:
   - `smart-crawler-macos-x64.tar.gz` for Intel Macs
   - `smart-crawler-macos-arm64.tar.gz` for Apple Silicon Macs (M1/M2/M3)
3. Extract and install:
   ```bash
   # Extract the downloaded file
   tar -xzf smart-crawler-macos-*.tar.gz
   
   # Move to a directory in your PATH
   sudo mv smart-crawler /usr/local/bin/
   
   # Make it executable
   chmod +x /usr/local/bin/smart-crawler
   ```

### Option B: DMG Package

1. Download `smart-crawler-[version].dmg` from the releases page
2. Open the DMG file
3. Copy `smart-crawler` to `/usr/local/bin/` or your preferred location
4. Make it executable:
   ```bash
   chmod +x /usr/local/bin/smart-crawler
   ```

### Option C: Build from Source

If you have Rust installed:
```bash
git clone https://github.com/pixlie/SmartCrawler.git
cd SmartCrawler
cargo build --release
# Binary will be in target/release/smart-crawler
```

## Step 2: Set Up WebDriver

SmartCrawler requires a WebDriver server to control a browser. Choose one:

### Option A: Firefox with GeckoDriver (Recommended)

1. **Install Firefox** (if not already installed):
   ```bash
   # Using Homebrew (recommended)
   brew install firefox
   
   # Or download from firefox.com
   ```

2. **Install GeckoDriver**:
   ```bash
   # Using Homebrew (recommended)
   brew install geckodriver
   
   # Or download manually from GitHub releases
   wget https://github.com/mozilla/geckodriver/releases/latest/download/geckodriver-v0.33.0-macos.tar.gz
   tar -xzf geckodriver-v0.33.0-macos.tar.gz
   sudo mv geckodriver /usr/local/bin/
   ```

### Option B: Chrome with ChromeDriver

1. **Install Chrome** (if not already installed):
   ```bash
   # Using Homebrew
   brew install google-chrome
   
   # Or download from chrome.com
   ```

2. **Install ChromeDriver**:
   ```bash
   # Using Homebrew
   brew install chromedriver
   
   # Or download manually - check your Chrome version first
   google-chrome --version
   # Then download matching version from Chrome for Testing
   ```

## Step 3: Handle macOS Security

macOS may block unsigned executables. If you get a security warning:

1. **Allow the binary to run**:
   ```bash
   # Remove quarantine attribute
   xattr -d com.apple.quarantine /usr/local/bin/smart-crawler
   
   # Or allow in System Preferences
   # System Preferences > Security & Privacy > General > Allow anyway
   ```

2. **For WebDriver binaries**:
   ```bash
   # If you downloaded manually
   xattr -d com.apple.quarantine /usr/local/bin/geckodriver
   xattr -d com.apple.quarantine /usr/local/bin/chromedriver
   ```

## Step 4: Test Your Setup

1. **Open Terminal**
2. **Start WebDriver** (choose one):
   ```bash
   # For Firefox (GeckoDriver)
   geckodriver --port 4444
   
   # For Chrome (ChromeDriver)
   chromedriver --port=4444
   ```
3. **Open a new Terminal window**
4. **Test SmartCrawler**:
   ```bash
   smart-crawler --link "https://example.com"
   ```

## Step 5: Run Your First Crawl

```bash
# Basic crawl
smart-crawler --link "https://example.com"

# Crawl with verbose output
smart-crawler --link "https://example.com" --verbose

# Crawl with template detection
smart-crawler --link "https://example.com" --template --verbose

# Crawl multiple sites
smart-crawler --link "https://example.com" --link "https://another.com"
```

## Troubleshooting

### "WebDriver connection failed"
- Ensure WebDriver is running on port 4444
- Check that the browser is installed
- Try restarting the WebDriver
- Verify no other application is using port 4444

### "Permission denied" errors
- Make sure the binary is executable:
  ```bash
  chmod +x /usr/local/bin/smart-crawler
  ```
- Check that `/usr/local/bin` is in your PATH:
  ```bash
  echo $PATH
  ```

### "smart-crawler: command not found"
- If you didn't install to `/usr/local/bin`, add the location to your PATH:
  ```bash
  export PATH=$PATH:/path/to/smart-crawler
  ```
- Or run with the full path:
  ```bash
  /path/to/smart-crawler --link "https://example.com"
  ```

### macOS Security Warnings
- Remove quarantine attributes:
  ```bash
  xattr -d com.apple.quarantine /usr/local/bin/smart-crawler
  ```
- Or go to System Preferences > Security & Privacy > General and click "Allow anyway"

### Port already in use
- Kill any existing WebDriver processes:
  ```bash
  pkill geckodriver
  pkill chromedriver
  ```

## Advanced Setup with Homebrew

If you use Homebrew, you can install everything in one go:

```bash
# Install browsers and WebDriver
brew install firefox geckodriver

# Or for Chrome
brew install google-chrome chromedriver

# Download SmartCrawler binary and install
# (Follow Option A above for binary installation)
```

## Next Steps

- Read the [CLI Options documentation](cli-options.md) for advanced usage
- Learn more about template detection for content pattern analysis
- Explore verbose mode for detailed HTML tree analysis

## Getting Help

If you encounter issues:

1. Check the [troubleshooting section](#troubleshooting) above
2. Visit the [GitHub Issues page](https://github.com/pixlie/SmartCrawler/issues)
3. Search for existing solutions or create a new issue
4. Include your macOS version, browser version, and error messages

## Additional Resources

- [Homebrew](https://brew.sh/) - Package manager for macOS
- [Firefox Download](https://www.firefox.com/)
- [Chrome Download](https://www.google.com/chrome/)
- [GeckoDriver Releases](https://github.com/mozilla/geckodriver/releases)
- [ChromeDriver Downloads](https://googlechromelabs.github.io/chrome-for-testing/)
- [Rust Installation](https://rustup.rs/) (if building from source)