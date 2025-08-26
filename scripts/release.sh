#!/bin/bash

# Moonflare Release Script
# This script helps create new releases by updating the version and creating a git tag

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -f "src/main.rs" ]]; then
    print_error "This script must be run from the Moonflare project root directory"
    exit 1
fi

# Check if git working directory is clean
if [[ -n $(git status --porcelain) ]]; then
    print_error "Working directory is not clean. Please commit or stash your changes first."
    git status --short
    exit 1
fi

# Check if we're on main branch
current_branch=$(git branch --show-current)
if [[ "$current_branch" != "main" ]]; then
    print_warning "You are not on the main branch (current: $current_branch)"
    read -p "Do you want to continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Aborting release"
        exit 1
    fi
fi

# Get current version from Cargo.toml
current_version=$(grep '^version = ' Cargo.toml | head -n1 | sed 's/version = "\(.*\)"/\1/')
print_info "Current version: $current_version"

# Get new version from user
echo
echo "Enter the new version number (without 'v' prefix):"
echo "Examples: 0.1.1, 0.2.0, 1.0.0, 0.1.0-beta.1"
read -p "New version: " new_version

# Validate version format (basic check)
if [[ ! $new_version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$ ]]; then
    print_error "Invalid version format. Please use semantic versioning (e.g., 1.0.0 or 1.0.0-beta.1)"
    exit 1
fi

# Check if tag already exists
if git rev-parse "v$new_version" >/dev/null 2>&1; then
    print_error "Tag v$new_version already exists"
    exit 1
fi

print_info "Updating version from $current_version to $new_version"

# Update Cargo.toml
sed -i.bak "s/^version = \"$current_version\"/version = \"$new_version\"/" Cargo.toml
rm Cargo.toml.bak

# Update Cargo.lock
cargo check --quiet

# Show changes
print_info "Changes to be committed:"
git diff --name-only

# Confirm with user
echo
read -p "Do you want to create the release? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_info "Aborting release. Reverting changes..."
    git checkout -- Cargo.toml Cargo.lock
    exit 1
fi

# Create commit
print_info "Creating release commit..."
git add Cargo.toml Cargo.lock
git commit -m "chore: release v$new_version"

# Create tag
print_info "Creating release tag..."
git tag -a "v$new_version" -m "Release v$new_version"

# Push changes
print_info "Pushing changes to origin..."
git push origin "$current_branch"
git push origin "v$new_version"

print_success "Release v$new_version created successfully!"
print_info "GitHub Actions will now build and publish the release automatically."
print_info "You can monitor the progress at: https://github.com/moonflare-dev/moonflare/actions"

echo
print_info "Release Summary:"
echo "  - Version: v$new_version"
echo "  - Tag: v$new_version"
echo "  - Branch: $current_branch"
echo "  - Commit: $(git rev-parse --short HEAD)"
echo
print_info "The release will be available at: https://github.com/moonflare-dev/moonflare/releases/tag/v$new_version"