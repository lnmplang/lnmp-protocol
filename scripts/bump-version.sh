#!/bin/bash
set -e

VERSION=$1
if [ -z "$VERSION" ]; then
  echo "Usage: ./bump-version.sh VERSION"
  echo "Example: ./bump-version.sh 0.6.0"
  exit 1
fi

echo "🔄 Bumping version to $VERSION..."

# Bump Rust workspace version
echo "📦 Updating Rust Cargo.toml..."
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Bump NPM packages
echo "📦 Updating NPM packages..."
cd sdk/js/packages/wasm-bindings
npm version $VERSION --no-git-tag-version --allow-same-version

cd ../lnmp
npm version $VERSION --no-git-tag-version --allow-same-version

cd ../../..

echo "✅ Version bumped to $VERSION"
echo "📝 Don't forget to update CHANGELOG.md!"
