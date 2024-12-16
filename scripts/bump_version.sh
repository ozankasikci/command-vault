#!/bin/bash

# Function to display usage
usage() {
    echo "Usage: $0 [major|minor|patch]"
    echo "Bump the version number in Cargo.toml"
    echo ""
    echo "Arguments:"
    echo "  major    Bump major version (X.y.z -> X+1.0.0)"
    echo "  minor    Bump minor version (x.Y.z -> x.Y+1.0)"
    echo "  patch    Bump patch version (x.y.Z -> x.y.Z+1)"
    exit 1
}

# Check if cargo-edit is installed
if ! command -v cargo-set-version &> /dev/null; then
    echo "cargo-edit is not installed. Installing..."
    cargo install cargo-edit
fi

# Validate arguments
if [ "$#" -ne 1 ]; then
    usage
fi

VERSION_TYPE=$1

case $VERSION_TYPE in
    major|minor|patch)
        # Get current version
        CURRENT_VERSION=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
        echo "Current version: $CURRENT_VERSION"

        # Bump version using cargo-set-version
        cargo set-version --bump $VERSION_TYPE
        
        # Update README.md with new version
        ./scripts/update_version.sh

        # Get new version
        NEW_VERSION=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
        echo "New version: $NEW_VERSION"

        # Stage changes
        git add Cargo.toml Cargo.lock
        git commit -m "chore: bump version to $NEW_VERSION"

        # Create tag
        git tag -a "v$NEW_VERSION" -m "Version $NEW_VERSION"

        # Push changes and tags
        echo "Pushing changes and tags..."
        git push && git push --tags

        echo "Version bump complete!"
        ;;
    *)
        usage
        ;;
esac
