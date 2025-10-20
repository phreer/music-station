# Music Station

A Rust-based HTTP server for managing and streaming FLAC music files with a CLI client for browsing.

## Features

- 🎵 Scan local music library folders
- 📊 Extract FLAC metadata (title, artist, album, duration)
- 🌐 REST API for music library access
- 🎧 Stream FLAC files over HTTP
- 💻 CLI client for browsing library
- ▶️ Audio playback directly from CLI client
- 🌐 Web client for browsing and managing tracks
- ✏️ Edit track metadata (title, artist, album)

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

### Using the Web Client

1. Start the server (see above)
2. Open your browser to `http://localhost:3000/web/index.html`
3. Browse your music library with a clean web interface
4. Click "Edit" on any track to update its metadata (title, artist, album)
5. Changes are saved directly to the FLAC files

## API Endpoints

- `GET /` - API version information
- `GET /tracks` - List all tracks (JSON array)
- `GET /tracks/:id` - Get track details by ID
- `PUT /tracks/:id` - Update track metadata (JSON: {title?, artist?, album?})
- `GET /stream/:id` - Stream FLAC audio file
- `GET /web/*` - Static web client files

## Project Structure

```
music-station/
├── src/
│   ├── main.rs           # Server entry point
│   ├── library.rs        # Music library scanner & FLAC parser
│   ├── server.rs         # HTTP API handlers
│   └── bin/
│       └── client.rs     # CLI client
├── static/
│   ├── index.html        # Web client UI
│   ├── styles.css        # Web client styles
│   └── app.js            # Web client JavaScript
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

# Run with debug logging (default)
cargo run -- --library /path/to/music

# Run with less verbose logging
RUST_LOG=info cargo run -- --library /path/to/music
```

### Debug Logging

The server includes comprehensive debug logging:
- All HTTP requests and responses
- File operations and metadata updates
- Performance metrics (request duration)
- Error diagnostics

See [DEBUG_LOGGING.md](DEBUG_LOGGING.md) for detailed logging configuration and troubleshooting guide.

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
- **metaflac** - FLAC metadata writing
- **rodio** - Audio playback (client)
- **serde** - JSON serialization
- **clap** - CLI argument parsing
- **reqwest** - HTTP client (for CLI)

## License

See LICENSE file for details.
