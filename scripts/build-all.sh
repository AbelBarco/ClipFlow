#!/bin/bash
# Multi-platform build script for ClipFlow

set -e

VERSION=${1:-$(cat package.json | grep '"version"' | head -1 | sed 's/.*"version": "\(.*\)".*/\1/')}

echo "🏗️  Building ClipFlow v$VERSION for all platforms..."

# Build frontend
echo "📦 Building frontend..."
pnpm run build

# Build for current platform
echo "🔨 Building for current platform..."
pnpm tauri build

# For cross-compilation, you would need:
# - Windows: cross (cargo install cross) + appropriate targets
# - macOS: osxcross or GitHub Actions macOS runners
# - Linux: docker or GitHub Actions Linux runners

echo ""
echo "✅ Build complete!"
echo "📁 Artifacts in: src-tauri/target/release/bundle/"
echo ""
echo "For cross-platform builds, use GitHub Actions workflows."