#!/bin/bash

# Claude-Flow Local Build Script
# This script builds the claude-flow package for local npx testing
# Usage: ./build-claude-flow.sh

set -e  # Exit on any error

echo "🚀 Claude-Flow Local Build Script"
echo "=================================="
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js >= 18"
    echo "   Visit: https://nodejs.org/"
    exit 1
fi

# Check if pnpm is installed
if ! command -v pnpm &> /dev/null; then
    echo "❌ pnpm is not installed. Please install pnpm >= 8"
    echo "   Run: npm install -g pnpm"
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo is not installed. Please install Rust"
    echo "   Visit: https://rustup.rs/"
    exit 1
fi

echo "✅ Prerequisites check passed"
echo ""

# Set environment variables for local development
export LOCAL_DEV_MODE=1
export VIBE_KANBAN_DEBUG=1

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf npx-cli/dist
rm -rf frontend/dist
rm -rf target/release
mkdir -p npx-cli/dist/macos-arm64
mkdir -p npx-cli/dist/linux-x64
mkdir -p npx-cli/dist/windows-x64

echo "✅ Clean complete"
echo ""

# Install dependencies
echo "📦 Installing dependencies..."
pnpm install
echo "✅ Dependencies installed"
echo ""

# Build frontend
echo "🔨 Building frontend..."
cd frontend
pnpm run build
cd ..
echo "✅ Frontend build complete"
echo ""

# Build Rust binaries
echo "🔨 Building Rust binaries..."
echo "   Building main server..."
cargo build --release --manifest-path Cargo.toml

echo "   Building MCP task server..."
cargo build --release --bin mcp_task_server --manifest-path Cargo.toml

echo "   Building review CLI..."
cargo build --release --bin review --manifest-path Cargo.toml

echo "✅ Rust binaries built"
echo ""

# Create distribution packages for multiple platforms
echo "📦 Creating distribution packages..."

# macOS ARM64
echo "   Creating macOS ARM64 package..."
cp target/release/server vibe-kanban-macos-arm64
zip -q vibe-kanban-macos-arm64.zip vibe-kanban-macos-arm64
rm -f vibe-kanban-macos-arm64
mv vibe-kanban-macos-arm64.zip npx-cli/dist/macos-arm64/vibe-kanban.zip

cp target/release/mcp_task_server vibe-kanban-mcp-macos-arm64
zip -q vibe-kanban-mcp-macos-arm64.zip vibe-kanban-mcp-macos-arm64
rm -f vibe-kanban-mcp-macos-arm64
mv vibe-kanban-mcp-macos-arm64.zip npx-cli/dist/macos-arm64/vibe-kanban-mcp.zip

cp target/release/review vibe-kanban-review-macos-arm64
zip -q vibe-kanban-review-macos-arm64.zip vibe-kanban-review-macos-arm64
rm -f vibe-kanban-review-macos-arm64
mv vibe-kanban-review-macos-arm64.zip npx-cli/dist/macos-arm64/vibe-kanban-review.zip

# Linux x64
echo "   Creating Linux x64 package..."
cp target/release/server vibe-kanban-linux-x64
zip -q vibe-kanban-linux-x64.zip vibe-kanban-linux-x64
rm -f vibe-kanban-linux-x64
mv vibe-kanban-linux-x64.zip npx-cli/dist/linux-x64/vibe-kanban.zip

cp target/release/mcp_task_server vibe-kanban-mcp-linux-x64
zip -q vibe-kanban-mcp-linux-x64.zip vibe-kanban-mcp-linux-x64
rm -f vibe-kanban-mcp-linux-x64
mv vibe-kanban-mcp-linux-x64.zip npx-cli/dist/linux-x64/vibe-kanban-mcp.zip

cp target/release/review vibe-kanban-review-linux-x64
zip -q vibe-kanban-review-linux-x64.zip vibe-kanban-review-linux-x64
rm -f vibe-kanban-review-linux-x64
mv vibe-kanban-review-linux-x64.zip npx-cli/dist/linux-x64/vibe-kanban-review.zip

# Windows x64
echo "   Creating Windows x64 package..."
cp target/release/server.exe vibe-kanban-windows-x64.exe 2>/dev/null || cp target/release/server vibe-kanban-windows-x64
zip -q vibe-kanban-windows-x64.zip vibe-kanban-windows-x64* 2>/dev/null || zip -q vibe-kanban-windows-x64.zip vibe-kanban-windows-x64
rm -f vibe-kanban-windows-x64*
mv vibe-kanban-windows-x64.zip npx-cli/dist/windows-x64/vibe-kanban.zip

cp target/release/mcp_task_server.exe vibe-kanban-mcp-windows-x64.exe 2>/dev/null || cp target/release/mcp_task_server vibe-kanban-mcp-windows-x64
zip -q vibe-kanban-mcp-windows-x64.zip vibe-kanban-mcp-windows-x64* 2>/dev/null || zip -q vibe-kanban-mcp-windows-x64.zip vibe-kanban-mcp-windows-x64
rm -f vibe-kanban-mcp-windows-x64*
mv vibe-kanban-mcp-windows-x64.zip npx-cli/dist/windows-x64/vibe-kanban-mcp.zip

cp target/release/review.exe vibe-kanban-review-windows-x64.exe 2>/dev/null || cp target/release/review vibe-kanban-review-windows-x64
zip -q vibe-kanban-review-windows-x64.zip vibe-kanban-review-windows-x64* 2>/dev/null || zip -q vibe-kanban-review-windows-x64.zip vibe-kanban-review-windows-x64
rm -f vibe-kanban-review-windows-x64*
mv vibe-kanban-review-windows-x64.zip npx-cli/dist/windows-x64/vibe-kanban-review.zip

echo "✅ Distribution packages created"
echo ""

# Update package.json for local development
echo "⚙️  Updating package.json for local development..."

# Backup original package.json
cp npx-cli/package.json npx-cli/package.json.backup 2>/dev/null || true

# Update package name and version for local testing
cat > npx-cli/package.json << 'EOF'
{
  "name": "claude-flow",
  "version": "0.0.1-alpha.0",
  "main": "index.js",
  "bin": {
    "claude-flow": "bin/cli.js"
  },
  "keywords": [],
  "author": "clauderun",
  "license": "MIT",
  "description": "Claude Flow - AI-powered development orchestration",
  "dependencies": {
    "adm-zip": "^0.5.16"
  },
  "files": [
    "bin",
    "dist"
  ]
}
EOF

echo "✅ Package.json updated"
echo ""

# Build NPX package
echo "📦 Building NPX package..."
cd npx-cli
npm pack
cd ..

echo ""
echo "✅ Build complete!"
echo ""
echo "📁 Files created:"
echo "   - npx-cli/claude-flow-0.0.1-alpha.0.tgz"
echo "   - Distribution packages in npx-cli/dist/"
echo ""
echo "🚀 To test locally with npx:"
echo "   Option 1: Link globally"
echo "     cd npx-cli"
echo "     npm link"
echo "     claude-flow"
echo ""
echo "   Option 2: Use npx with local package"
echo "     cd npx-cli"
echo "     npx claude-flow@alpha"
echo ""
echo "   Option 3: Install from tarball"
echo "     cd npx-cli"
echo "     npm install -g claude-flow-0.0.1-alpha.0.tgz"
echo "     claude-flow"
echo ""
echo "✨ Claude-Flow is ready for local testing!"
