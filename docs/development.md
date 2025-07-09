# Development Guide

This guide covers how to set up a development environment, build from source, contribute to the project, and understand the codebase.

## 🛠️ Development Environment Setup

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **WebDriver**: Firefox (GeckoDriver) or Chrome (ChromeDriver) for testing

### Building from Source

```bash
# Clone the repository
git clone https://github.com/pixlie/SmartCrawler.git
cd SmartCrawler

# Build in release mode
cargo build --release

# The binary will be in target/release/smart-crawler
```

### Development Build

```bash
# Build in development mode (faster compilation, includes debug info)
cargo build

# Run directly with cargo
cargo run -- --link "https://example.com"
```

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in a specific module
cargo test html_parser::tests
```

### Test Structure

The project includes several types of tests:

- **Unit tests**: Located in each module (`src/` files)
- **Integration tests**: In the `tests/` directory
- **Real-world tests**: For testing against actual websites (normally ignored)

### Running Real-World Tests

```bash
# Run real-world tests (requires WebDriver)
cargo test --test real_world_tests -- --ignored
```

## 🔧 Development Commands

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run linter
cargo clippy

# Run linter with warnings as errors
cargo clippy -- -D warnings
```

### Debug Mode

```bash
# Run with debug logging
RUST_LOG=debug cargo run -- --link "https://example.com"

# Run with specific module logging
RUST_LOG=smart_crawler::html_parser=debug cargo run -- --link "https://example.com"
```

### Profiling and Performance

```bash
# Build with profiling
cargo build --release --features profiling

# Run with timing information
RUST_LOG=info cargo run --release -- --link "https://example.com"
```

## 📁 Project Structure

```
SmartCrawler/
├── src/
│   ├── main.rs              # Main application entry point
│   ├── lib.rs               # Library exports
│   ├── browser.rs           # WebDriver browser integration
│   ├── cli.rs               # Command-line argument parsing
│   ├── html_parser.rs       # HTML parsing and tree building
│   ├── storage.rs           # URL storage and duplicate detection
│   ├── template_detection.rs # Template pattern detection
│   └── utils.rs             # Utility functions
├── tests/
│   └── real_world_tests.rs  # Integration tests
├── docs/                    # Documentation
├── CLAUDE.md               # Development workflow guide
└── Cargo.toml              # Project dependencies
```

## 🏗️ Architecture Overview

### Core Components

1. **CLI Interface** (`cli.rs`): Parses command-line arguments
2. **Browser Integration** (`browser.rs`): WebDriver integration for page loading
3. **HTML Parser** (`html_parser.rs`): Parses HTML into structured tree
4. **Storage System** (`storage.rs`): Manages URLs and duplicate detection
5. **Template Detection** (`template_detection.rs`): Identifies content patterns
6. **Utilities** (`utils.rs`): Common helper functions

### Data Flow

```
CLI Arguments → Browser → HTML Source → HTML Parser → Storage → Duplicate Analysis → Output
                                                    ↓
                                            Template Detection
```

### Key Features

- **Domain-level duplicate detection**: Identifies similar content across pages
- **Template pattern recognition**: Converts variable content to template patterns
- **HTML tree filtering**: Shows structured view with duplicate marking
- **Multi-URL crawling**: Processes multiple URLs with smart discovery

## 🤝 Contributing

### Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork**:
   ```bash
   git clone https://github.com/your-username/SmartCrawler.git
   cd SmartCrawler
   ```
3. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

### Development Workflow

Follow the workflow described in `CLAUDE.md`:

1. **Create a new branch** for each feature/fix
2. **Add tests** for any new functionality
3. **Run formatters and linters** before committing
4. **Write descriptive commit messages**
5. **Push to your fork** and create a pull request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Address all clippy warnings (`cargo clippy`)
- Write comprehensive tests for new features
- Document public APIs with doc comments
- Use meaningful variable and function names

### Commit Message Format

```
type: brief description

- Detailed explanation of changes
- Reference any related issues
- Include any breaking changes

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

### Pull Request Process

1. **Ensure tests pass**: Run `cargo test`
2. **Check code quality**: Run `cargo fmt` and `cargo clippy`
3. **Update documentation**: Add or update relevant docs
4. **Describe your changes**: Write a clear PR description
5. **Link related issues**: Reference any GitHub issues

### Code Review Guidelines

- Be constructive and respectful
- Focus on code quality and maintainability
- Test the changes locally when possible
- Ask questions if something isn't clear

## 🐛 Debugging

### Common Issues

**Build failures**:
- Ensure Rust is up to date: `rustup update`
- Check dependencies: `cargo update`

**Test failures**:
- Ensure WebDriver is running for integration tests
- Check that target websites are accessible

**WebDriver issues**:
- Verify WebDriver version matches browser version
- Check that port 4444 is available
- Ensure browser is installed and accessible

### Debug Tools

```bash
# Check dependencies
cargo tree

# Update dependencies
cargo update

# Clean build artifacts
cargo clean

# Verbose build output
cargo build --verbose
```

## 📚 Learning Resources

### Rust Resources

- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust Standard Library](https://doc.rust-lang.org/std/)

### WebDriver Resources

- [WebDriver Specification](https://w3c.github.io/webdriver/)
- [GeckoDriver Documentation](https://firefox-source-docs.mozilla.org/testing/geckodriver/)
- [ChromeDriver Documentation](https://chromedriver.chromium.org/)

### HTML Parsing

- [scraper crate documentation](https://docs.rs/scraper/)
- [HTML5 Specification](https://html.spec.whatwg.org/)

## 🔒 Security

### Security Considerations

- Never commit API keys or secrets
- Validate all user inputs
- Respect robots.txt and website terms of service
- Use HTTPS for all network requests where possible

### Reporting Security Issues

If you discover a security vulnerability:

1. **Do NOT** create a public GitHub issue
2. Email the maintainers directly
3. Provide detailed information about the vulnerability
4. Allow time for a fix before public disclosure

## 📈 Performance

### Optimization Tips

- Use `--release` builds for performance testing
- Profile with `perf` or similar tools
- Monitor memory usage during development
- Test with various website sizes and structures

### Benchmarking

```bash
# Build optimized version
cargo build --release

# Time execution
time target/release/smart-crawler --link "https://example.com"

# Profile memory usage
valgrind target/release/smart-crawler --link "https://example.com"
```

## 🎯 Future Improvements

Areas for contribution:

- **Performance optimization**: Faster HTML parsing and duplicate detection
- **Additional output formats**: JSON, CSV, XML export options
- **Enhanced filtering**: More sophisticated duplicate detection algorithms
- **UI improvements**: Better progress reporting and error messages
- **Platform support**: Windows-specific optimizations
- **Documentation**: More examples and use cases

## 📞 Getting Help

- **GitHub Issues**: For bug reports and feature requests
- **Discussions**: For general questions and ideas
- **Code Review**: For feedback on contributions
- **Documentation**: For clarification on usage

Remember to search existing issues before creating new ones!

---

This development guide is continuously updated. If you find any information missing or outdated, please contribute improvements!