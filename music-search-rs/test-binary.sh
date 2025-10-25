#!/bin/bash
# Quick test script for the music_search binary

echo "=== Music Search Binary Test ==="
echo ""
echo "Binary location: target/release/music_search"
echo "Binary size: $(ls -lh target/release/music_search | awk '{print $5}')"
echo ""
echo "The music_search binary is an interactive CLI tool."
echo "To use it, run:"
echo ""
echo "  ./target/release/music_search"
echo ""
echo "Or install it system-wide:"
echo ""
echo "  cargo install --path . --bin music_search"
echo ""
echo "For detailed usage instructions, see BINARY_USAGE.md"
echo ""
echo "=== Example Session ==="
echo ""
echo "1. Select music service (1=NetEase, 2=QQ Music)"
echo "2. Enter search query (e.g., '告白气球' or '周杰伦')"
echo "3. Select a song from the search results"
echo "4. Choose lyrics type (0 to download all)"
echo "5. Lyrics will be saved to ./lyrics/ directory"
echo ""
echo "=== Features ==="
echo ""
echo "✓ Search songs on NetEase Cloud Music and QQ Music"
echo "✓ Download original, translated, and transliteration lyrics"
echo "✓ Save lyrics in standard LRC format"
echo "✓ Interactive command-line interface"
echo "✓ Automatic filename sanitization"
echo ""

# Check if binary exists
if [ -f "target/release/music_search" ]; then
    echo "✓ Binary is ready to use!"
    exit 0
else
    echo "✗ Binary not found. Run: cargo build --bin music_search --release"
    exit 1
fi
