# Music Station

A Rust-based HTTP server for managing and streaming FLAC music files with a CLI client for browsing.

## Features

- 🎵 Scan local music library folders
- 📊 Extract FLAC metadata (title, artist, album, duration)
- 🌐 REST API for music library access
- 🎧 Stream FLAC files over HTTP
- 💻 CLI client for browsing library
- ▶️ Audio playback directly from CLI client

## Quick Start

### Prerequisites

- Rust toolchain (Edition 2024)
- A folder with FLAC music files

### Running the Server

```bash
# Using command-line argument
cargo run -- --library /path/to/music/folder

# Or using environment variable
export MUSIC_LIBRARY_PATH=/path/to/music
cargo run

# Specify custom port (default is 3000)
cargo run -- --library /path/to/music --port 8080
```

### Using the CLI Client

```bash
# List all tracks in the library
cargo run --bin music-client

# List with custom server URL
cargo run --bin music-client -- --server http://localhost:3000 list

# Get detailed information about a specific track
cargo run --bin music-client -- info <track-id>

# Play a specific track
cargo run --bin music-client -- play <track-id>

# Play all tracks in the library
cargo run --bin music-client -- play-all
```

## API Endpoints

- `GET /` - API version information
- `GET /tracks` - List all tracks (JSON array)
- `GET /tracks/:id` - Get track details by ID
- `GET /stream/:id` - Stream FLAC audio file

## Project Structure

```
music-station/
├── src/
│   ├── main.rs           # Server entry point
│   ├── library.rs        # Music library scanner & FLAC parser
│   ├── server.rs         # HTTP API handlers
│   └── bin/
│       └── client.rs     # CLI client
├── Cargo.toml
└── README.md
```

## Development

```bash
# Build the project
cargo build

# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Architecture

The server:
1. Scans the specified library folder on startup
2. Parses FLAC metadata using Symphonia
3. Stores track information in memory (thread-safe with `Arc<RwLock>`)
4. Serves REST API via Axum on port 3000 (configurable)

The client:
1. Connects to the server via HTTP
2. Fetches and displays track information
3. Provides formatted output for easy browsing
4. Streams and plays audio using Rodio audio library

## Current Limitations

- Only FLAC format is supported
- Full file loaded into memory for streaming (no chunked streaming)
- Library scan happens only on startup (no hot-reload)
- No recursive directory scanning (only top-level files)

## Dependencies

- **axum** - Web framework
- **tokio** - Async runtime
- **symphonia** - Audio decoding (server)
- **rodio** - Audio playback (client)
- **serde** - JSON serialization
- **clap** - CLI argument parsing
- **reqwest** - HTTP client (for CLI)

## License

See LICENSE file for details.
