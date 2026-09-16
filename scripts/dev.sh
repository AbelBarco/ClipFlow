#!/bin/bash
# Development environment setup script

set -e

echo "🚀 Setting up ClipFlow development environment..."

# Check for required tools
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ $1 is not installed. Please install it first."
        exit 1
    fi
    echo "✅ $1 found"
}

check_tool "node"
check_tool "pnpm"
check_tool "rustup"
check_tool "cargo"

# Install Rust targets
echo "📦 Installing Rust targets..."
rustup target add x86_64-pc-windows-msvc 2>/dev/null || true
rustup target add x86_64-apple-darwin 2>/dev/null || true
rustup target add aarch64-apple-darwin 2>/dev/null || true
rustup target add x86_64-unknown-linux-gnu 2>/dev/null || true

# Install Node dependencies
echo "📦 Installing Node dependencies..."
pnpm install

# Install Tauri CLI
echo "📦 Installing Tauri CLI..."
cargo install tauri-cli --version "^2.0" 2>/dev/null || cargo install tauri-cli --version "^2.0" --force

# Platform-specific dependencies
case "$(uname -s)" in
    Linux*)
        echo "🐧 Linux detected - installing system dependencies..."
        if command -v apt-get &> /dev/null; then
            sudo apt-get update
            sudo apt-get install -y \
                libwebkit2gtk-4.1-dev \
                libayatana-appindicator3-dev \
                librsvg2-dev \
                libssl-dev \
                libsqlite3-dev \
                tesseract-ocr \
                libtesseract-dev \
                build-essential \
                curl \
                wget \
                file \
                libxcb-render0-dev \
                libxcb-shape0-dev \
                libxcb-xfixes0-dev \
                libspeechd-dev \
                libxkbcommon-dev \
                libxrandr-dev \
                libxi-dev \
                libx11-dev \
                libxcomposite-dev \
                libxcursor-dev \
                libxdamage-dev \
                libxext-dev \
                libxfixes-dev \
                libxinerama-dev \
                libxrender-dev \
                libxtst-dev
        elif command -v dnf &> /dev/null; then
            sudo dnf install -y \
                webkit2gtk4.1-devel \
                libayatana-appindicator-gtk3-devel \
                librsvg2-devel \
                openssl-devel \
                sqlite-devel \
                tesseract \
                tesseract-devel \
                gcc \
                gcc-c++ \
                make \
                pkg-config
        elif command -v pacman &> /dev/null; then
            sudo pacman -S --needed \
                webkit2gtk-4.1 \
                libayatana-appindicator \
                librsvg \
                openssl \
                sqlite \
                tesseract \
                base-devel
        fi
        ;;
    Darwin*)
        echo "🍎 macOS detected - checking for Xcode..."
        if ! xcode-select -p &> /dev/null; then
            echo "❌ Xcode Command Line Tools not found. Run: xcode-select --install"
            exit 1
        fi
        # Install tesseract via Homebrew if available
        if command -v brew &> /dev/null; then
            brew install tesseract
        fi
        ;;
    *)
        echo "⚠️  Unknown OS - skipping system dependencies"
        ;;
esac

echo "✅ Development environment ready!"
echo ""
echo "To start development:"
echo "  pnpm tauri dev"