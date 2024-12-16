#!/bin/bash

# Get the new version from Cargo.toml
NEW_VERSION=$(grep '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')

# Update the version in README.md install instructions
# First try to replace X.Y.Z pattern
sed -i '' "s/vX\.Y\.Z/v$NEW_VERSION/g" README.md
# Then try to replace any existing version numbers
sed -i '' -E "s/v[0-9]+\.[0-9]+\.[0-9]+/v$NEW_VERSION/g" README.md

echo "Updated version to $NEW_VERSION in:"
echo "- Cargo.toml"
echo "- README.md"
