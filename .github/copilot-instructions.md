# Music Station - AI Coding Agent Instructions

## Project Overview
Music Station is a Rust-based HTTP server that scans a music library folder, extracts metadata from FLAC files, and provides a REST API for browsing and streaming music. Currently supports FLAC format only.

**Architecture**: Client-server with REST API
- Server: `music-station` binary scans library and serves HTTP API
- Client: `music-client` CLI tool for browsing the library

## Development Environment
- **Language**: Rust (Edition 2024)
- **Build Tool**: Cargo
- **Runtime**: Tokio async runtime
- **Web Framework**: Axum with tower-http middleware
- **Audio**: Symphonia for FLAC parsing

## Key Commands
```bash
# Run the server (requires music library path)
cargo run -- --library /path/to/music/folder
# Or set environment variable
MUSIC_LIBRARY_PATH=/path/to/music cargo run

# Run the CLI client (connects to server on localhost:3000 by default)
cargo run --bin music-client                    # List all tracks
cargo run --bin music-client -- list            # List all tracks
cargo run --bin music-client -- info <track-id> # Show track details
cargo run --bin music-client -- play <track-id> # Play a specific track
cargo run --bin music-client -- play-all        # Play all tracks

# Development
cargo check      # Quick compile check
cargo clippy     # Linting
cargo fmt        # Auto-format code
cargo build      # Build without running
cargo test       # Run tests
```

## Project Structure
```
music-station/
├── src/
│   ├── main.rs           # Server entry: CLI parsing, library init, HTTP server
│   ├── library.rs        # Music library scanner and FLAC metadata parser
│   ├── server.rs         # Axum HTTP routes and handlers
│   └── bin/
│       └── client.rs     # CLI client for browsing library
├── Cargo.toml            # Two binaries: music-station, music-client
└── .github/
    └── copilot-instructions.md
```

## Architecture & Data Flow

### Server Startup Flow
1. `main.rs` parses CLI args (`--library`, `--port`)
2. Creates `MusicLibrary` instance with library path
3. Calls `library.scan()` to recursively scan folder for `.flac` files
4. Each FLAC file is parsed with Symphonia to extract metadata
5. Tracks stored in `Arc<RwLock<Vec<Track>>>` for concurrent access
6. Axum router created with shared library state
7. HTTP server starts on `0.0.0.0:3000` (or specified port)

### API Endpoints
- `GET /` - API version info
- `GET /tracks` - List all tracks (returns JSON array)
- `GET /tracks/:id` - Get single track details
- `GET /stream/:id` - Stream FLAC file with proper headers

### Client Flow
1. `music-client` sends HTTP request to server
2. Parses JSON response into local `Track` struct
3. Displays formatted output to terminal
4. For playback: downloads audio stream and plays using Rodio

## Critical Implementation Patterns

### Async/Await with Tokio
- All I/O operations use async/await (file reading, HTTP)
- `#[tokio::main]` macro on main functions
- `tokio::fs` for async file operations
- `Arc<RwLock<T>>` for shared mutable state across async tasks

### FLAC Metadata Extraction (library.rs)
```rust
// Symphonia pattern: create media source → probe format → read metadata
let mss = MediaSourceStream::new(Box::new(file), Default::default());
let mut hint = Hint::new();
hint.with_extension("flac");
let probed = symphonia::default::get_probe()
    .format(&hint, mss, &Default::default(), &MetadataOptions::default())?;
```
- Metadata tags: `TITLE`, `ARTIST`, `ALBUM` (case-sensitive)
- Duration calculated from time base and frame count
- Track ID generated using MD5 hash of file path

### Axum State Pattern (server.rs)
```rust
#[derive(Clone)]
pub struct AppState {
    pub library: MusicLibrary,
}
// Router uses .with_state(state) to share library across handlers
// Handlers extract with State(state): State<AppState>
```

### Error Handling
- Use `anyhow::Result<T>` for application errors
- `.context("helpful message")` to add error context
- HTTP handlers return `Result<T, StatusCode>` for proper error responses
- Tracing for logging errors during library scan

### Streaming Audio Files (Server)
- Files loaded into memory with `tokio::fs::read_to_end`
- Returned with `Content-Type: audio/flac` header
- `Content-Disposition: inline` for browser playback

### Audio Playback (Client)
- Uses Rodio library for cross-platform audio output
- Downloads full audio stream via reqwest
- Decodes FLAC using Rodio's Decoder
- Creates OutputStream and Sink for playback
- `sink.sleep_until_end()` blocks until playback completes

## Dependencies & Their Roles
- **axum**: Web framework for REST API
- **tokio**: Async runtime (required for axum)
- **tower-http**: CORS middleware
- **symphonia**: Audio decoding (FLAC metadata extraction on server)
- **rodio**: Audio playback (FLAC playback in client)
- **serde**: JSON serialization for Track struct
- **clap**: CLI argument parsing (derive + env features)
- **anyhow**: Ergonomic error handling
- **tracing**: Structured logging
- **md5**: Generate track IDs from file paths
- **reqwest**: HTTP client for music-client (with streaming support)

## Testing Approach
- Unit tests: Test metadata parsing with sample FLAC files
- Integration tests: Test API endpoints with test server
- Mock `MusicLibrary` for server handler tests
- Use `#[tokio::test]` for async tests

## Common Development Tasks

### Adding New Audio Format Support
1. Add format feature to symphonia in Cargo.toml
2. Update `library.rs` file extension check
3. Add format-specific metadata tag mappings
4. Update Content-Type header in streaming endpoint

### Adding New API Endpoints
1. Define handler function in `server.rs`
2. Extract state/path params with axum extractors
3. Add route to router in `create_router()`
4. Update client.rs if client needs to call it

### Modifying Track Metadata
1. Update `Track` struct in `library.rs` (add `#[derive(Serialize)]`)
2. Add metadata extraction in `parse_flac_file()`
3. Duplicate `Track` struct in `client.rs` (or consider shared types)

## Performance Notes
- Library scanning is synchronous on startup (blocking)
- Consider incremental scanning for large libraries (>10k files)
- File streaming loads entire file into memory - add chunked streaming for large files
- RwLock allows concurrent reads, single writer for library updates

## VS Code Integration
- Custom workspace theme (brown/orange titlebar)
- Use Rust Analyzer for code completion and inline errors
- Run tasks: Create tasks.json for common cargo commands
