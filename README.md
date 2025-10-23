# Music Station

A Rust-based HTTP server for managing and streaming music files with a CLI client and web interface.

## Features

- 🎵 Scan local music library folders
- 🎼 Support for FLAC and MP3 audio formats
- 📊 Extract metadata (title, artist, album, duration, cover art)
- 🌐 REST API for music library access
- 🎧 Stream audio files over HTTP with range request support
- 💻 CLI client for browsing library
- ▶️ Audio playback directly from CLI client
- 🌐 Web client for browsing and managing tracks
- ✏️ Edit track metadata (title, artist, album, genre, year, etc.)
- 🖼️ Cover art management (view, add, remove)
- 📝 Lyrics support with plain text and LRC (synchronized) formats
- 💾 SQLite database for persistent lyrics storage

## Quick Start

### Prerequisites

- Rust toolchain (Edition 2024)
- A folder with FLAC or MP3 music files

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

**Tracks:**
- `GET /` - API version information
- `GET /tracks` - List all tracks (JSON array)
- `GET /tracks/:id` - Get track details by ID
- `PUT /tracks/:id` - Update track metadata (JSON: {title?, artist?, album?})
- `GET /stream/:id` - Stream FLAC audio file

**Lyrics:**
- `GET /lyrics/:id` - Get lyrics for a track
- `PUT /lyrics/:id` - Upload/update lyrics (JSON: {content, format?, language?, source?})
- `DELETE /lyrics/:id` - Delete lyrics for a track

**Albums & Artists:**
- `GET /albums` - List all albums
- `GET /albums/:name` - Get album details
- `GET /artists` - List all artists
- `GET /artists/:name` - Get artist details

**Web Client:**
- `GET /web/*` - Static web client files

For detailed lyrics API usage, see [LYRICS_GUIDE.md](LYRICS_GUIDE.md).

## Project Structure

```
music-station/
├── src/
│   ├── main.rs           # Server entry point
│   ├── library.rs        # Music library scanner & audio parser (FLAC/MP3)
│   ├── lyrics/           # Lyrics management module
│   │   ├── mod.rs        # Lyrics database
│   │   ├── fetcher.rs    # Lyrics fetching API traits
│   │   └── providers.rs  # Example lyrics providers
│   ├── server.rs         # HTTP API handlers
│   └── bin/
│       └── client.rs     # CLI client
├── static/
│   ├── index.html        # Web client UI
│   ├── styles.css        # Web client styles
│   └── app.js            # Web client JavaScript
├── Cargo.toml
├── README.md
├── LYRICS_GUIDE.md       # Lyrics feature documentation
└── LYRICS_API.md         # Lyrics fetching API documentation
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
2. Parses audio metadata using Symphonia (FLAC and MP3 support)
3. Stores track information in memory (thread-safe with `Arc<RwLock>`)
4. Serves REST API via Axum on port 3000 (configurable)
5. Supports HTTP range requests for efficient audio streaming

The client:
1. Connects to the server via HTTP
2. Fetches and displays track information
3. Provides formatted output for easy browsing
4. Streams and plays audio using Rodio audio library

## Current Limitations

- Library scan happens only on startup (no hot-reload)
- No recursive directory scanning (only top-level files)
- MP3 metadata editing supports standard ID3v2 tags (custom fields limited)

## Dependencies

- **axum** - Web framework
- **tokio** - Async runtime
- **symphonia** - Audio decoding (FLAC and MP3 support)
- **metaflac** - FLAC metadata writing
- **id3** - MP3 metadata reading and writing
- **rodio** - Audio playback (client)
- **serde** - JSON serialization
- **clap** - CLI argument parsing
- **reqwest** - HTTP client (for CLI)
- **sqlx** - Database driver for lyrics storage
- **chrono** - Date and time handling

## License

See LICENSE file for details.
