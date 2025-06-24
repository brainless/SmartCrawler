#!/bin/bash

# SmartCrawler Release Script
# This script automates the release process including version bumping and tagging

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to get current version from Cargo.toml
get_current_version() {
    grep '^version = ' "$CARGO_TOML" | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Function to update version in Cargo.toml
update_version() {
    local new_version="$1"
    
    # Create backup
    cp "$CARGO_TOML" "$CARGO_TOML.backup"
    
    # Update version in Cargo.toml
    sed -i.tmp "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    rm "$CARGO_TOML.tmp" 2>/dev/null || true
    
    # Update version in RPM metadata
    sed -i.tmp "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    rm "$CARGO_TOML.tmp" 2>/dev/null || true
    
    log_success "Updated version to $new_version in Cargo.toml"
}

# Function to validate semantic version
validate_version() {
    local version="$1"
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+(\.[a-zA-Z0-9]+)*)?(\+[a-zA-Z0-9]+(\.[a-zA-Z0-9]+)*)?$ ]]; then
        log_error "Invalid semantic version: $version"
        log_error "Valid format: MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]"
        exit 1
    fi
}

# Function to bump version
bump_version() {
    local bump_type="$1"
    local current_version
    current_version=$(get_current_version)
    
    # Parse current version
    local major minor patch prerelease
    IFS='.' read -r major minor patch_and_pre <<< "$current_version"
    
    # Handle prerelease/build metadata
    if [[ $patch_and_pre =~ ^([0-9]+)(-.*)?(\+.*)?$ ]]; then
        patch="${BASH_REMATCH[1]}"
        prerelease="${BASH_REMATCH[2]}"
    else
        patch="$patch_and_pre"
        prerelease=""
    fi
    
    # Bump version based on type
    case "$bump_type" in
        "major")
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        "minor")
            minor=$((minor + 1))
            patch=0
            ;;
        "patch")
            patch=$((patch + 1))
            ;;
        *)
            log_error "Invalid bump type: $bump_type. Use major, minor, or patch."
            exit 1
            ;;
    esac
    
    echo "$major.$minor.$patch"
}

# Function to create git tag
create_tag() {
    local version="$1"
    local tag="v$version"
    
    # Check if tag already exists
    if git rev-parse "$tag" >/dev/null 2>&1; then
        log_error "Tag $tag already exists!"
        exit 1
    fi
    
    # Create annotated tag
    git tag -a "$tag" -m "Release version $version"
    log_success "Created git tag: $tag"
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS] <VERSION_OR_BUMP_TYPE>

ARGUMENTS:
    <VERSION_OR_BUMP_TYPE>    Either a specific version (e.g., 1.2.3) or bump type (major, minor, patch)

OPTIONS:
    -h, --help               Show this help message
    -n, --dry-run           Show what would be done without making changes
    -p, --push              Push the tag to remote after creating it
    -c, --check             Check current version and git status

EXAMPLES:
    $0 1.2.3                # Set version to 1.2.3
    $0 patch                # Bump patch version (1.2.3 -> 1.2.4)
    $0 minor                # Bump minor version (1.2.3 -> 1.3.0)
    $0 major                # Bump major version (1.2.3 -> 2.0.0)
    $0 --dry-run patch       # Show what patch bump would do
    $0 --push 1.2.3          # Release version 1.2.3 and push tag

EOF
}

# Parse command line arguments
DRY_RUN=false
PUSH_TAG=false
CHECK_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -n|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -p|--push)
            PUSH_TAG=true
            shift
            ;;
        -c|--check)
            CHECK_ONLY=true
            shift
            ;;
        -*)
            log_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
        *)
            VERSION_ARG="$1"
            shift
            ;;
    esac
done

# Change to project root
cd "$PROJECT_ROOT"

# Check git status
if ! git diff-index --quiet HEAD --; then
    log_warn "You have uncommitted changes. Please commit or stash them first."
    git status --short
    if [[ "$CHECK_ONLY" == "false" ]]; then
        exit 1
    fi
fi

current_version=$(get_current_version)
log_info "Current version: $current_version"

if [[ "$CHECK_ONLY" == "true" ]]; then
    log_info "Git status:"
    git status --short
    exit 0
fi

# Validate arguments
if [[ -z "$VERSION_ARG" ]]; then
    log_error "Please specify a version or bump type"
    show_usage
    exit 1
fi

# Determine new version
if [[ "$VERSION_ARG" =~ ^(major|minor|patch)$ ]]; then
    # Bump version
    new_version=$(bump_version "$VERSION_ARG")
    log_info "Bumping $VERSION_ARG version: $current_version -> $new_version"
else
    # Explicit version
    new_version="$VERSION_ARG"
    validate_version "$new_version"
    log_info "Setting version to: $new_version"
fi

# Dry run mode
if [[ "$DRY_RUN" == "true" ]]; then
    log_info "DRY RUN MODE - No changes will be made"
    log_info "Would update Cargo.toml version to: $new_version"
    log_info "Would create git tag: v$new_version"
    if [[ "$PUSH_TAG" == "true" ]]; then
        log_info "Would push tag to remote"
    fi
    exit 0
fi

# Confirm with user
read -p "Proceed with release $new_version? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    log_info "Release cancelled"
    exit 0
fi

# Update version
update_version "$new_version"

# Run tests to make sure everything works
log_info "Running tests..."
if ! cargo test; then
    log_error "Tests failed! Rolling back changes."
    mv "$CARGO_TOML.backup" "$CARGO_TOML"
    exit 1
fi

# Remove backup
rm "$CARGO_TOML.backup"

# Commit version bump
git add "$CARGO_TOML"
git commit -m "chore: bump version to $new_version"
log_success "Committed version bump"

# Create git tag
create_tag "$new_version"

# Push tag if requested
if [[ "$PUSH_TAG" == "true" ]]; then
    log_info "Pushing tag to remote..."
    git push origin "v$new_version"
    git push origin HEAD
    log_success "Pushed tag v$new_version to remote"
    log_info "GitHub Actions will now build and publish the release"
fi

log_success "Release $new_version completed!"
log_info "To trigger the release workflow, push the tag:"
log_info "  git push origin v$new_version"