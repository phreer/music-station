# Music Station - AI Coding Agent Instructions

## Project Overview
Music Station is a Rust-based HTTP server that scans a music library folder, extracts metadata from audio files, and provides a REST API for browsing, streaming, and managing music.

**Architecture**: Client-server with REST API + Web UI
- **Server**: `music-station` binary — scans library, serves HTTP API, persists data in SQLite
- **CLI Client**: `music-client` binary — browsing and playback via CLI
- **Web Client**: Vanilla JS single-page app (no framework) served from `static/`
- **External Clients**: GTK desktop client ([music-station-client-gtk](https://github.com/phreer/music-station-client-gtk))
- **Subcrate**: `music-search-rs/` — local crate for NetEase and QQ Music search/lyrics APIs

## Style Guidelines
- Design consistent UI/UX for web client (clean, minimal, responsive)
- Follow Rust's official style guidelines (rustfmt)
- Use idiomatic Rust patterns (Result, Option, iterators, error handling with anyhow)
- Write modular, reusable functions with clear responsibilities
- Document public functions and complex logic with comments, avoid trivial and unnecessary comments
- Always keep code and documentation in sync
- Don't write apparent documentation

## Key Commands
```bash
# Run the server (requires music library path)
cargo run -- --library /path/to/music/folder
# Or set environment variable
MUSIC_LIBRARY_PATH=/path/to/music cargo run

# Run the CLI client (connects to server on localhost:3000 by default)
cargo run --bin music-client -- list            # List all tracks
cargo run --bin music-client -- info <track-id> # Show track details
cargo run --bin music-client -- play <track-id> # Play a specific track
cargo run --bin music-client -- play-all        # Play all tracks

# Utility binaries
cargo run --bin export-lyrics                   # Export lyrics to files
cargo run --bin migrate-track-ids               # Migrate track IDs

# Development
cargo check      # Quick compile check
cargo clippy     # Linting
cargo fmt        # Auto-format code
cargo build      # Build without running
cargo test       # Run tests
```

## Architecture & Data Flow

### Server Startup Flow
1. `main.rs` parses CLI args (`--library`, `--port`) via clap
2. Creates `MusicLibrary` instance with library path
3. Calls `library.scan()` to recursively scan folder for supported audio files (.flac, .mp3, .ogg, .m4a)
4. Each file dispatched to format-specific handler via `AudioFile` trait in `audio.rs`
5. Metadata extracted with Symphonia; tracks stored in `Arc<RwLock<Vec<Track>>>`
6. Initializes 4 SQLite databases in `<library>/.music-station/` directory:
   - `lyrics.db` — lyrics storage with format detection
   - `playlists.db` — server-side playlist persistence
   - `stats.db` — play count tracking
   - `favorites.db` — favorite artists
7. Loads lyrics flags and play counts into in-memory track state
8. Creates lyrics providers (NetEase, QQ Music) from `music-search-rs`
9. Axum router created with shared `AppState`, HTTP server starts on `0.0.0.0:3000`

### Source File Map
| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point, CLI args, DB init, server startup |
| `src/library.rs` | `MusicLibrary`, `Track`, `Album`, `Artist` structs and scanning |
| `src/audio.rs` | `AudioFile` trait with `FlacFile`, `Mp3File`, `OggFile`, `M4aFile` implementations |
| `src/server.rs` | HTTP handlers, `AppState`, `create_router()` with 29 routes |
| `src/lyrics.rs` | `LyricDatabase`, lyrics format detection (Plain/LRC/LRC-Word) |
| `src/lyrics/fetcher.rs` | `LyricsProvider` trait, `LyricsAggregator` for multi-provider fallback |
| `src/lyrics/music_search_provider.rs` | NetEase & QQ Music lyrics provider implementations |
| `src/playlist.rs` | `PlaylistDatabase`, server-side CRUD for playlists |
| `src/stats.rs` | `StatsDatabase`, play count persistence |
| `src/favorites.rs` | `FavoritesDatabase`, favorite artists persistence |
| `src/bin/client.rs` | CLI client binary |
| `static/` | New web client (Vue 3 + Vite, built output) |
| `static-legacy/` | Legacy web client (vanilla JS SPA) |
| `web/` | New web client source (Vue 3, TypeScript, Pinia, Naive UI) |
| `music-search-rs/` | Local crate: NetEase Music and QQ Music search/lyrics APIs |

### API Endpoints

**Tracks:**
- `GET /` — API version info
- `GET /tracks` — List all tracks (JSON array)
- `GET /tracks/:id` — Get single track details
- `PUT /tracks/:id` — Update track metadata
- `POST /tracks/:id/play` — Increment play count

**Streaming:**
- `GET /stream/:id` — Stream audio with HTTP Range support (Content-Type per format)

**Cover Art:**
- `GET /cover/:id` — Fetch embedded cover art
- `POST /cover/:id` — Upload cover (multipart/form-data)
- `DELETE /cover/:id` — Remove cover art

**Lyrics:**
- `GET /lyrics/:id` — Get stored lyrics for track
- `PUT /lyrics/:id` — Upload/update lyrics (multipart or JSON)
- `DELETE /lyrics/:id` — Remove lyrics
- `GET /lyrics/search?q=...&provider=...` — Search lyrics online
- `GET /lyrics/fetch/:provider/:song_id` — Fetch from specific provider

**Albums & Artists:**
- `GET /albums` — List all albums with track counts and durations
- `GET /albums/:name` — Get specific album with tracks
- `GET /artists` — List all artists with album/track counts
- `GET /artists/:name` — Get specific artist with albums

**Playlists (server-side persistent):**
- `GET /playlists` — List all playlists
- `POST /playlists` — Create playlist (`{name, description?}`)
- `GET /playlists/:id` — Get specific playlist
- `PUT /playlists/:id` — Update playlist
- `DELETE /playlists/:id` — Delete playlist
- `POST /playlists/:id/tracks/:track_id` — Add track
- `DELETE /playlists/:id/tracks/:track_id` — Remove track

**Favorites:**
- `GET /favorites/artists` — List all favorite artists
- `PUT /favorites/artists/:name` — Add artist to favorites
- `DELETE /favorites/artists/:name` — Remove artist from favorites

**Statistics:**
- `GET /stats` — Library stats (total tracks, albums, artists, duration, size, play counts)

**Static Files:**
- `GET /web/*` — Serves `static/` directory

## Critical Implementation Patterns

### Audio Format Handling (audio.rs)
```rust
// AudioFile trait — each format implements read/write/cover operations
pub trait AudioFile {
    fn parse_metadata(&self) -> Result<TrackMetadata>;
    fn write_metadata(&self, metadata: &TrackMetadata) -> Result<()>;
    fn get_cover(&self) -> Result<Option<CoverArt>>;
    fn set_cover(&self, cover: &CoverArt) -> Result<()>;
    fn remove_cover(&self) -> Result<()>;
}
```
- `get_audio_file_handler(path)` dispatches to: `FlacFile`, `Mp3File`, `OggFile`, `M4aFile`
- **FLAC**: metaflac for writing, Symphonia for reading; Vorbis comment tags
- **MP3**: id3 crate for ID3v2 tags (`TIT2`, `TPE1`, `TALB`, etc.)
- **OGG Vorbis**: Symphonia; Vorbis comments (same keys as FLAC)
- **M4A/AAC**: mp4ameta crate; iTunes-style tags
- Duration extracted via Symphonia frame counting for all formats
- Track ID: MD5 hash of file path

### Axum State Pattern (server.rs)
```rust
pub struct AppState {
    pub library: MusicLibrary,
    pub lyrics_db: LyricDatabase,
    pub playlist_db: PlaylistDatabase,
    pub stats_db: StatsDatabase,
    pub netease_provider: Option<Arc<NetEaseLyricsProvider>>,
    pub qqmusic_provider: Option<Arc<QQMusicLyricsProvider>>,
}
// Handlers extract with State(state): State<AppState>
```

### SQLite Database Pattern
- Three separate `.db` files in `<library>/.music-station/`
- sqlx with `runtime-tokio` + `sqlite` features
- Tables created with `CREATE TABLE IF NOT EXISTS` on startup
- Async query methods returning `anyhow::Result`

### Lyrics System
- **Formats**: `Plain` (raw text), `Lrc` (line-level `[mm:ss.xx]`), `LrcWord` (word-level `word(offset,duration)`)
- **Timestamp variants**: Standard LRC uses `[mm:ss.xx]`; offset LRC uses `[offset_ms,duration_ms]` (common from NetEase/QQ Music). See [`LYRICS_GUIDE.md`](./LYRICS_GUIDE.md) for format details and examples.
- **Auto-detection** via regex pattern matching on content
- **Providers**: `LyricsProvider` async trait with `NetEaseLyricsProvider` and `QQMusicLyricsProvider`
- **Aggregation**: `LyricsAggregator` tries multiple providers with fallback
- `Track.has_lyrics: bool` flag synced between library and database

### Async/Await with Tokio
- All I/O uses async/await; `#[tokio::main]` on main functions
- `Arc<RwLock<T>>` for shared mutable state across async tasks
- `tokio::fs` for async file operations

### Error Handling
- `anyhow::Result<T>` for application errors with `.context("message")`
- HTTP handlers return `Result<T, StatusCode>` for proper error responses
- Tracing (`tracing` crate) for structured logging

### HTTP Range Streaming
- RFC 7233 range requests for partial content delivery
- Returns 206 with `Content-Range` header for range requests, 200 for full file
- Content-Type per format: `audio/flac`, `audio/mpeg`, `audio/ogg`, `audio/mp4`
- `Accept-Ranges: bytes` header; `Content-Disposition: inline` for browser playback

### Client Audio Playback
- Rodio library for cross-platform audio output
- Downloads full stream via reqwest, decodes with Rodio's Decoder
- `sink.sleep_until_end()` blocks until playback completes

## Dependencies & Their Roles
- **axum** (+ multipart): Web framework for REST API
- **tokio**: Async runtime
- **tower-http**: CORS, tracing, and static file serving middleware
- **symphonia**: Audio metadata reading (FLAC, MP3, OGG, AAC/M4A)
- **metaflac**: FLAC metadata writing (Vorbis comments)
- **id3**: MP3 metadata writing (ID3v2 tags)
- **mp4ameta**: M4A/AAC metadata writing (iTunes-style tags)
- **sqlx** (sqlite): Async SQLite database (lyrics, playlists, stats)
- **music-search-rs**: Local subcrate for NetEase/QQ Music search and lyrics APIs
- **rodio**: Audio playback (CLI client)
- **serde** / **serde_json**: JSON serialization
- **clap**: CLI argument parsing (derive + env features)
- **anyhow**: Ergonomic error handling
- **tracing**: Structured logging
- **md5**: Track ID generation from file paths
- **reqwest**: HTTP client (CLI client, with streaming)
- **uuid**: Playlist ID generation
- **chrono**: Timestamp handling
- **base64**: Cover art encoding
- **regex**: Lyrics format detection

## Testing
- `cargo test` runs all tests
- `tests/lyric_format_tests.rs` — tests for lyrics format parsing and detection
- Use `#[tokio::test]` for async tests
- Shell scripts: `test_lyrics.sh`, `test_word_level_format.sh` for integration testing

## Common Development Tasks

### Adding New Audio Format Support
1. Add format feature to symphonia in `Cargo.toml`
2. Create new struct implementing `AudioFile` trait in `audio.rs`
3. Register in `get_audio_file_handler()` dispatcher
4. Add Content-Type mapping in streaming endpoint

### Adding New API Endpoints
1. Define async handler function in `server.rs`
2. Extract state/path/query params with axum extractors
3. Add route to router in `create_router()`
4. Update `client.rs` if CLI client needs it

### Adding a New Lyrics Provider
1. Implement `LyricsProvider` trait in `src/lyrics/`
2. Add provider instance to `AppState` in `main.rs`
3. Register in `search_lyrics` / `fetch_lyrics_from_provider` handlers

## Performance Notes
- Library scanning is synchronous on startup (blocking)
- File streaming supports HTTP Range requests for efficient seeking
- RwLock allows concurrent reads, single writer for library updates
- SQLite databases are local to library folder for portability
