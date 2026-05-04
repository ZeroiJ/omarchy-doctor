#!/bin/bash
# Install omadoctor system-wide

set -e

VERSION="0.1.0"

echo "🔧 Building omadoctor..."
cargo build --release

echo "📁 Creating directories..."
sudo mkdir -p /usr/share/omadoctor/fixes

echo "📦 Installing binary..."
sudo cp target/release/omadoctor /usr/bin/omadoctor

echo "📋 Installing fixes..."
sudo cp fixes/*.toml /usr/share/omadoctor/fixes/

echo "🏷️  Creating version file..."
echo "$VERSION" | sudo tee /usr/share/omadoctor/VERSION > /dev/null

echo "✅ omadoctor installed!"
echo ""
echo "Usage:"
echo "  omadoctor          - Launch interactive TUI"
echo "  omadoctor --scan   - Run non-interactive scan"
echo "  omadoctor --help   - Show help"
echo "  omadoctor --version - Show version"
echo ""
echo "Fix database will auto-update from GitHub on startup."
