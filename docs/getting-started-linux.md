# Getting Started on Linux

This guide will help you set up SmartCrawler on Linux systems.

## Prerequisites

- Linux distribution (Ubuntu, Debian, CentOS, Fedora, etc.)
- Root/sudo access for installation
- Internet connection for downloads

## Step 1: Install SmartCrawler

### Option A: Download Pre-built Binary (Recommended)

1. Go to the [SmartCrawler releases page](https://github.com/pixlie/SmartCrawler/releases)
2. Download `smart-crawler-linux-x64.tar.gz`
3. Extract and install:
   ```bash
   # Extract the downloaded file
   tar -xzf smart-crawler-linux-x64.tar.gz
   
   # Move to a directory in your PATH
   sudo mv smart-crawler /usr/local/bin/
   
   # Make it executable
   chmod +x /usr/local/bin/smart-crawler
   ```

### Option B: Package Installers

#### Ubuntu/Debian (DEB Package)
```bash
# Download and install the DEB package
wget https://github.com/pixlie/SmartCrawler/releases/latest/download/smart-crawler-[version].deb
sudo dpkg -i smart-crawler-[version].deb

# Install dependencies if needed
sudo apt-get install -f
```

#### RHEL/CentOS/Fedora (RPM Package)
```bash
# Download and install the RPM package
wget https://github.com/pixlie/SmartCrawler/releases/latest/download/smart-crawler-[version].rpm
sudo rpm -i smart-crawler-[version].rpm

# Or with dnf/yum
sudo dnf install smart-crawler-[version].rpm
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

#### Ubuntu/Debian
```bash
# Install Firefox
sudo apt update
sudo apt install firefox

# Install GeckoDriver
wget https://github.com/mozilla/geckodriver/releases/latest/download/geckodriver-v0.33.0-linux64.tar.gz
tar -xzf geckodriver-v0.33.0-linux64.tar.gz
sudo mv geckodriver /usr/local/bin/
chmod +x /usr/local/bin/geckodriver
```

#### CentOS/RHEL/Fedora
```bash
# Install Firefox
sudo dnf install firefox

# Install GeckoDriver
wget https://github.com/mozilla/geckodriver/releases/latest/download/geckodriver-v0.33.0-linux64.tar.gz
tar -xzf geckodriver-v0.33.0-linux64.tar.gz
sudo mv geckodriver /usr/local/bin/
chmod +x /usr/local/bin/geckodriver
```

#### Arch Linux
```bash
# Install Firefox and GeckoDriver
sudo pacman -S firefox geckodriver
```

### Option B: Chrome with ChromeDriver

#### Ubuntu/Debian
```bash
# Install Chrome
wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | sudo apt-key add -
echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" | sudo tee /etc/apt/sources.list.d/google-chrome.list
sudo apt update
sudo apt install google-chrome-stable

# Install ChromeDriver
# First check Chrome version
google-chrome --version

# Download matching ChromeDriver version
CHROME_VERSION=$(google-chrome --version | cut -d' ' -f3 | cut -d'.' -f1-3)
wget https://chromedriver.storage.googleapis.com/LATEST_RELEASE_${CHROME_VERSION}
CHROMEDRIVER_VERSION=$(cat LATEST_RELEASE_${CHROME_VERSION})
wget https://chromedriver.storage.googleapis.com/${CHROMEDRIVER_VERSION}/chromedriver_linux64.zip
unzip chromedriver_linux64.zip
sudo mv chromedriver /usr/local/bin/
chmod +x /usr/local/bin/chromedriver
```

#### CentOS/RHEL/Fedora
```bash
# Install Chrome
sudo dnf install google-chrome-stable

# Install ChromeDriver (follow similar steps as Ubuntu above)
```

## Step 3: Test Your Setup

1. **Open a terminal**
2. **Start WebDriver** (choose one):
   ```bash
   # For Firefox (GeckoDriver)
   geckodriver --port 4444
   
   # For Chrome (ChromeDriver)
   chromedriver --port=4444
   ```
3. **Open a new terminal**
4. **Test SmartCrawler**:
   ```bash
   smart-crawler --link "https://example.com"
   ```

## Step 4: Run Your First Crawl

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

### Browser/WebDriver issues
- Check if browser is installed correctly:
  ```bash
  firefox --version
  google-chrome --version
  ```
- Verify WebDriver is accessible:
  ```bash
  geckodriver --version
  chromedriver --version
  ```

### Port already in use
- Kill any existing WebDriver processes:
  ```bash
  pkill geckodriver
  pkill chromedriver
  ```
- Check what's using port 4444:
  ```bash
  sudo netstat -tlnp | grep 4444
  ```

### Missing dependencies
- Install missing libraries:
  ```bash
  # Ubuntu/Debian
  sudo apt install libssl-dev pkg-config
  
  # CentOS/RHEL/Fedora
  sudo dnf install openssl-devel pkgconfig
  ```

## Distribution-Specific Notes

### Ubuntu/Debian
- Use `apt` for package management
- Firefox ESR is available via `firefox-esr` package
- Chrome installation requires adding Google's repository

### CentOS/RHEL/Fedora
- Use `dnf` or `yum` for package management
- EPEL repository may be needed for some packages
- Chrome is available through Google's repository

### Arch Linux
- Use `pacman` for package management
- Both Firefox and GeckoDriver are available in official repositories
- Chrome is available in AUR as `google-chrome`

### Alpine Linux
- Use `apk` for package management
- May need additional setup for glibc compatibility

## Next Steps

- Read the [CLI Options documentation](cli-options.md) for advanced usage
- Learn more about template detection for content pattern analysis
- Explore verbose mode for detailed HTML tree analysis

## Getting Help

If you encounter issues:

1. Check the [troubleshooting section](#troubleshooting) above
2. Visit the [GitHub Issues page](https://github.com/pixlie/SmartCrawler/issues)
3. Search for existing solutions or create a new issue
4. Include your Linux distribution, browser version, and error messages

## Additional Resources

- [Firefox Download](https://www.firefox.com/)
- [Chrome Download](https://www.google.com/chrome/)
- [GeckoDriver Releases](https://github.com/mozilla/geckodriver/releases)
- [ChromeDriver Downloads](https://googlechromelabs.github.io/chrome-for-testing/)
- [Rust Installation](https://rustup.rs/) (if building from source)

### Distribution-Specific Resources
- [Ubuntu Packages](https://packages.ubuntu.com/)
- [Debian Packages](https://packages.debian.org/)
- [Fedora Packages](https://packages.fedoraproject.org/)
- [Arch Linux AUR](https://aur.archlinux.org/)