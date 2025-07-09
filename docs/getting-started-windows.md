# Getting Started on Windows

This guide will help you set up SmartCrawler on Windows systems.

## Prerequisites

- Windows 10 or later
- Administrator access for installation
- Internet connection for downloads

## Step 1: Install SmartCrawler

### Option A: Download Pre-built Binary (Recommended)

1. Go to the [SmartCrawler releases page](https://github.com/pixlie/SmartCrawler/releases)
2. Download the latest `smart-crawler-windows-x64.zip`
3. Extract the ZIP file to a folder (e.g., `C:\SmartCrawler\`)
4. **Optional**: Add the folder to your system PATH:
   - Press `Win + X` and select "System"
   - Click "Advanced system settings"
   - Click "Environment Variables"
   - Under "System variables", find and select "Path", then click "Edit"
   - Click "New" and add your SmartCrawler folder path
   - Click "OK" to save

### Option B: MSI Installer

1. Download `smart-crawler-[version].msi` from the releases page
2. Double-click the MSI file to install
3. Follow the installation wizard
4. SmartCrawler will be automatically added to your PATH

### Option C: Build from Source

If you have Rust installed:
```powershell
git clone https://github.com/pixlie/SmartCrawler.git
cd SmartCrawler
cargo build --release
# Binary will be in target\release\smart-crawler.exe
```

## Step 2: Set Up WebDriver

SmartCrawler requires a WebDriver server to control a browser. Choose one:

### Option A: Firefox with GeckoDriver (Recommended)

1. **Install Firefox** (if not already installed):
   - Download from [firefox.com](https://www.firefox.com/)
   - Run the installer

2. **Install GeckoDriver**:
   - Download `geckodriver.exe` from [Mozilla GeckoDriver releases](https://github.com/mozilla/geckodriver/releases)
   - Extract the file and place it in:
     - The same folder as `smart-crawler.exe`, OR
     - A folder in your system PATH (e.g., `C:\Windows\System32`)

### Option B: Chrome with ChromeDriver

1. **Install Chrome** (if not already installed):
   - Download from [chrome.com](https://www.google.com/chrome/)
   - Run the installer

2. **Install ChromeDriver**:
   - Check your Chrome version: Go to `chrome://version/` in Chrome
   - Download the matching ChromeDriver from [Chrome for Testing](https://googlechromelabs.github.io/chrome-for-testing/)
   - Extract `chromedriver.exe` and place it in:
     - The same folder as `smart-crawler.exe`, OR
     - A folder in your system PATH

## Step 3: Test Your Setup

1. **Open Command Prompt or PowerShell**
2. **Start WebDriver** (choose one):
   ```cmd
   # For Firefox (GeckoDriver)
   geckodriver.exe --port 4444
   
   # For Chrome (ChromeDriver)
   chromedriver.exe --port=4444
   ```
3. **Open a new Command Prompt/PowerShell window**
4. **Test SmartCrawler**:
   ```cmd
   smart-crawler --link "https://example.com"
   ```

## Step 4: Run Your First Crawl

```cmd
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

### "'smart-crawler' is not recognized"
- If you didn't add SmartCrawler to PATH, run it with the full path:
  ```cmd
  C:\SmartCrawler\smart-crawler.exe --link "https://example.com"
  ```
- Or add the folder to your PATH (see Step 1)

### "Permission denied" errors
- Run Command Prompt as Administrator
- Check that the executable has proper permissions

### Port already in use
- Kill any existing WebDriver processes:
  ```cmd
  taskkill /F /IM geckodriver.exe
  taskkill /F /IM chromedriver.exe
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
4. Include your Windows version, browser version, and error messages

## Additional Resources

- [Firefox Download](https://www.firefox.com/)
- [Chrome Download](https://www.google.com/chrome/)
- [GeckoDriver Releases](https://github.com/mozilla/geckodriver/releases)
- [ChromeDriver Downloads](https://googlechromelabs.github.io/chrome-for-testing/)
- [Rust Installation](https://rustup.rs/) (if building from source)