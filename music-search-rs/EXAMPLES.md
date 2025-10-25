# Running the Examples

## Build Examples

```bash
# Build debug versions
cargo build --examples

# Build release versions (optimized, smaller binaries)
cargo build --release --examples
```

## Run Examples

### Using Cargo

```bash
# Run NetEase Cloud Music example
cargo run --example netease_search

# Run QQ Music example
cargo run --example qqmusic_search

# Run in release mode (faster)
cargo run --release --example netease_search
cargo run --release --example qqmusic_search
```

### Running Binaries Directly

```bash
# Debug binaries
./target/debug/examples/netease_search
./target/debug/examples/qqmusic_search

# Release binaries (recommended - smaller and faster)
./target/release/examples/netease_search
./target/release/examples/qqmusic_search
```

## Example Output

### NetEase Cloud Music

The example will:
1. Search for songs matching "告白气球"
2. Display top 5 results with artist and album info
3. Search for albums by "周杰伦"
4. Display top 3 album results
5. Fetch and display lyrics for a sample song

### QQ Music

The example will:
1. Search for songs matching "告白气球"
2. Display top 5 results with artist and album info
3. Search for playlists containing "周杰伦"
4. Display top 3 playlist results with play counts
5. Fetch details for a specific song

## Binary Sizes

- **Debug builds**: ~64MB per example (includes debug symbols)
- **Release builds**: ~4.8MB per example (optimized)

## Requirements

- Network connection (examples make real API calls)
- No authentication required for basic searches
- Cookie can be provided for authenticated features

## Troubleshooting

If you encounter network errors:
- Check your internet connection
- The music service APIs may have rate limits
- Try again after a short delay

If compilation fails:
- Run `cargo clean` and try again
- Ensure you have Rust 1.70+ installed
- Check that all dependencies are available
