#!/bin/bash
# Simple script to test the web demo locally

set -e

echo "🚀 Building RISC-V Emulator Web Demo"
echo

# Check dependencies
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "📦 Adding WASM target..."
    rustup target add wasm32-unknown-unknown
fi

# Build WASM package
echo "🔨 Building WASM package..."
wasm-pack build --target web --out-dir demo/web/pkg

echo "✅ Build complete!"
echo

# Start local server
if command -v python3 &> /dev/null; then
    echo "🌐 Starting local server at http://localhost:8000"
    echo "📄 Open http://localhost:8000 in your browser"
    echo "🛑 Press Ctrl+C to stop"
    echo
    cd demo/web
    python3 -m http.server 8000
elif command -v python &> /dev/null; then
    echo "🌐 Starting local server at http://localhost:8000"
    echo "📄 Open http://localhost:8000 in your browser"
    echo "🛑 Press Ctrl+C to stop"
    echo
    cd demo/web
    python -m SimpleHTTPServer 8000
else
    echo "⚠️  Python not found. Please start a local HTTP server in demo/web/"
    echo "📂 Built files are in: demo/web/"
    echo "📦 WASM package is in: demo/web/pkg/"
fi