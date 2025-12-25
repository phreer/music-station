# Music Station - AI Coding Agent Instructions

## Project Overview
Music Station is a Rust-based HTTP server that scans a music library folder, extracts metadata from audio files, and provides a REST API for browsing and streaming music.

**Architecture**: Client-server with REST API + Web UI
- Server: `music-station` binary scans library and serves HTTP API
- CLI Client: `music-client` CLI tool for browsing and playing music
- Web Client: Single-page web app with tabbed interface for browsing tracks, albums, artists, and statistics
- Desktop/Mobile Clients

## Development Environment
- **Language**: Rust (Edition 2024)
- **Build Tool**: Cargo
- **Runtime**: Tokio async runtime
- **Web Framework**: Axum with tower-http middleware

## Style Guidelines
- Follow Rust's official style guidelines (rustfmt)
- Use idiomatic Rust patterns (Result, Option, iterators, error handling with anyhow)
- Write modular, reusable functions with clear responsibilities
- Document public functions and complex logic with comments, avoid trivial and unnecessary comments
- Always keep code and documentation in sync

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

## Architecture & Data Flow

### Server Startup Flow
1. `main.rs` parses CLI args (`--library`, `--port`)
2. Creates `MusicLibrary` instance with library path
3. Calls `library.scan()` to recursively scan folder for `.flac` files
4. Each FLAC file is parsed with Symphonia to extract metadata
5. Tracks stored in `Arc<RwLock<Vec<Track>>>` for concurrent access
6. On-demand grouping by album/artist using HashMap aggregation
7. Axum router created with shared library state
8. HTTP server starts on `0.0.0.0:3000` (or specified port)

### API Endpoints

**Track Management:**
- `GET /` - API version info
- `GET /tracks` - List all tracks (returns JSON array)
- `GET /tracks/:id` - Get single track details
- `PUT /tracks/:id` - Update track metadata (body: {title?, artist?, album?})
- `GET /stream/:id` - Stream FLAC file with HTTP Range support for partial content delivery

**Album & Artist Browsing:**
- `GET /albums` - List all albums with track counts and durations
- `GET /albums/:name` - Get specific album with all its tracks
- `GET /artists` - List all artists with album/track counts
- `GET /artists/:name` - Get specific artist with all albums

**Library Statistics:**
- `GET /stats` - Get library statistics (total tracks, albums, artists, duration, size)

**Web Client:**
- `GET /web/*` - Static web client files (HTML, CSS, JS)

### Web Client Flow
1. User navigates to `http://localhost:3000/web/index.html`
2. Client loads with 5 tabs: Tracks, Albums, Artists, Playlists, Stats
3. **Tracks Tab**: Lists all tracks with edit and add-to-playlist functionality
4. **Albums Tab**: Shows albums grouped by name, expandable to view tracks
5. **Artists Tab**: Displays artists with nested album/track information
6. **Playlists Tab**: Manage custom playlists - create, view, play, and delete
7. **Stats Tab**: Shows library statistics with visual cards
8. Click album/artist cards to expand and view details
9. Edit track metadata via modal dialog (updates FLAC file)
10. Create playlists and add tracks from any view
11. Play entire playlists or individual tracks from playlists

### Client Flow (CLI)
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

### FLAC Metadata Writing (library.rs)
```rust
// metaflac pattern: read tags → modify → save
let mut tag = metaflac::Tag::read_from_path(path)?;
tag.set_vorbis("TITLE", vec![title.clone()]);
tag.save()?;
```
- Uses metaflac crate for writing (Symphonia is read-only)
- Updates Vorbis comments in FLAC files
- Re-parses file after write to update in-memory state

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
- Supports HTTP Range requests (RFC 7233) for partial content delivery
- Parses `Range: bytes=start-end` header to stream file segments
- Returns 206 Partial Content with `Content-Range` header for range requests
- Returns 200 OK with full file for non-range requests
- Includes `Accept-Ranges: bytes` header to advertise range support
- Uses `tokio::fs::File::seek()` for efficient range reads
- Enables audio seeking in web player without downloading entire file
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
- **tower-http**: CORS middleware and static file serving
- **symphonia**: Audio decoding (FLAC metadata extraction on server)
- **metaflac**: FLAC metadata writing (for editing track info)
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
- File streaming supports HTTP Range requests for efficient partial content delivery
- RwLock allows concurrent reads, single writer for library updates
- Playlists stored in browser localStorage (client-side only, not synced to server)

## Playlist System
- **Storage**: Client-side only, using browser localStorage
- **Features**: Create, rename, delete playlists; add/remove tracks; play playlists
- **Data Structure**: JSON array with playlist objects containing id, name, description, tracks[], createdAt
- **Integration**: Add-to-playlist button on all track rows; play entire playlist or individual tracks
- **Limitations**: Not synced across devices/browsers; data persists per browser only
