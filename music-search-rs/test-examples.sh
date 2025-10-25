#!/bin/bash
# Test runner for music-search-rs examples

set -e

echo "======================================"
echo "  Music Search RS - Example Tests"
echo "======================================"
echo ""

# Build examples
echo "Building examples..."
cargo build --examples --quiet

echo "✓ Examples built successfully"
echo ""

# List available examples
echo "Available examples:"
echo "  1. netease_search - NetEase Cloud Music search demo"
echo "  2. qqmusic_search - QQ Music search demo"
echo ""

echo "To run an example:"
echo "  cargo run --example netease_search"
echo "  cargo run --example qqmusic_search"
echo ""

echo "Or run the compiled binary directly:"
echo "  ./target/debug/examples/netease_search"
echo "  ./target/debug/examples/qqmusic_search"
echo ""

echo "Note: Examples make real API calls to NetEase and QQ Music."
echo "      Network connection required."
