# Release Management Guide

This document explains how to create releases for SmartCrawler, including version management, building artifacts, and publishing releases.

## Overview

SmartCrawler uses an automated GitHub Actions workflow to build releases for multiple platforms:

- **Linux**: x86_64 and ARM64 binaries, DEB and RPM packages
- **macOS**: x86_64 and ARM64 (Apple Silicon) binaries, DMG installer
- **Windows**: x86_64 binary, MSI installer

## Quick Release Process

### 1. Using the Release Script (Recommended)

The easiest way to create a release is using the provided release script:

```bash
# Bump patch version (e.g., 0.1.0 -> 0.1.1)
./scripts/release.sh patch --push

# Bump minor version (e.g., 0.1.0 -> 0.2.0)  
./scripts/release.sh minor --push

# Bump major version (e.g., 0.1.0 -> 1.0.0)
./scripts/release.sh major --push

# Set specific version
./scripts/release.sh 1.2.3 --push
```

The script will:
1. Update the version in `Cargo.toml`
2. Run tests to ensure everything works
3. Commit the version bump
4. Create a git tag (e.g., `v1.2.3`)
5. Push the tag to trigger the release workflow

### 2. Manual Release Process

If you prefer to do it manually:

```bash
# 1. Update version in Cargo.toml
vim Cargo.toml

# 2. Commit the version change
git add Cargo.toml
git commit -m "chore: bump version to 1.2.3"

# 3. Create and push tag
git tag -a v1.2.3 -m "Release version 1.2.3"
git push origin v1.2.3
git push origin main
```

## Release Script Usage

The release script (`scripts/release.sh`) provides several options:

### Basic Commands

```bash
# Check current version and git status
./scripts/release.sh --check

# Dry run to see what would happen
./scripts/release.sh --dry-run patch

# Create release and push immediately
./scripts/release.sh 1.2.3 --push
```

### Version Bumping

The script supports semantic versioning with three bump types:

- `major`: Breaking changes (1.2.3 → 2.0.0)
- `minor`: New features, backward compatible (1.2.3 → 1.3.0)  
- `patch`: Bug fixes, backward compatible (1.2.3 → 1.2.4)

### Prerelease Versions

You can create prerelease versions:

```bash
./scripts/release.sh 1.2.3-alpha.1
./scripts/release.sh 1.2.3-beta.2
./scripts/release.sh 1.2.3-rc.1
```

## GitHub Actions Workflow

The release is handled by `.github/workflows/release.yml` which:

### 1. Creates Release
- Generates changelog from git commits
- Creates GitHub release with release notes
- Handles prerelease detection (alpha, beta, rc)

### 2. Builds Binaries
Builds for all supported platforms:
- Linux x86_64 and ARM64
- macOS x86_64 and ARM64
- Windows x86_64

### 3. Creates Native Installers
- **Windows**: MSI installer using WiX Toolset
- **macOS**: DMG package using create-dmg
- **Linux**: DEB package using cargo-deb
- **Linux**: RPM package using cargo-rpm

### 4. Uploads Artifacts
All binaries and installers are uploaded to the GitHub release.

## Installer Details

### Windows MSI Installer
- Created using cargo-wix and WiX Toolset
- Installs to `Program Files\SmartCrawler`
- Adds to Windows PATH
- Creates Start Menu shortcut
- Supports upgrade and uninstall

### macOS DMG Package
- Created using create-dmg
- Drag-and-drop installer interface
- Code-signed (if certificates are available)
- Compatible with both Intel and Apple Silicon Macs

### Linux DEB Package
- Compatible with Debian, Ubuntu, and derivatives
- Installs to `/usr/bin/smart-crawler`
- Includes man page and documentation
- Automatic dependency resolution

### Linux RPM Package
- Compatible with Red Hat, CentOS, Fedora, and derivatives
- Same installation paths as DEB package
- Supports RPM-based package managers

## Version Management

### Semantic Versioning
SmartCrawler follows [Semantic Versioning (SemVer)](https://semver.org/):

- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality, backward compatible
- **PATCH**: Bug fixes, backward compatible

### Version Configuration
Version metadata is configured in `Cargo.toml`:

```toml
[package]
version = "0.1.0"

[package.metadata.deb]
version = "0.1.0"

[package.metadata.rpm]  
version = "0.1.0"
```

The release script automatically updates all version references.

## Troubleshooting

### Common Issues

1. **"Tag already exists" error**
   ```bash
   # Delete local tag
   git tag -d v1.2.3
   
   # Delete remote tag (if needed)
   git push origin --delete v1.2.3
   ```

2. **Build failures in GitHub Actions**
   - Check the Actions tab in GitHub repository
   - Common causes: test failures, dependency issues, missing secrets

3. **Installer build failures**
   - Windows: WiX Toolset installation issues
   - macOS: create-dmg dependency problems
   - Linux: Missing system dependencies for cargo-deb/cargo-rpm

### Manual Workflow Trigger

You can manually trigger the release workflow:

1. Go to GitHub → Actions → Release
2. Click "Run workflow"
3. Select branch and enter tag name
4. Click "Run workflow"

## Release Checklist

Before creating a release:

- [ ] All tests pass locally: `cargo test`
- [ ] Code is properly formatted: `cargo fmt --check`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Documentation is up to date
- [ ] CHANGELOG.md is updated (optional)
- [ ] Version number follows semantic versioning
- [ ] No uncommitted changes in git

After creating a release:

- [ ] Verify all artifacts were uploaded
- [ ] Test installers on different platforms
- [ ] Update documentation if needed
- [ ] Announce release (if applicable)

## Permissions and Environment Variables

The workflow requires:

- `contents: write` permission (automatically granted to the workflow)
- `GITHUB_TOKEN`: Automatically provided by GitHub

Optional secrets for enhanced functionality:
- `APPLE_CERTIFICATE`: For macOS code signing (optional)
- `APPLE_CERTIFICATE_PASSWORD`: For macOS code signing (optional)

The workflow uses the modern `gh` CLI instead of deprecated GitHub Actions for reliability.

## Support

If you encounter issues with the release process:

1. Check the GitHub Actions logs
2. Verify all dependencies are installed
3. Ensure proper permissions for repository
4. Review this documentation
5. Open an issue in the repository

---

For more information about the project structure and development, see [CLAUDE.md](CLAUDE.md).