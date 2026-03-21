# Music Station

A Rust-based HTTP server for managing and streaming music files with a CLI client and web interface.

## Features

- 🎵 Scan local music library folders
- 🎼 Support for FLAC, MP3, OGG Vorbis, and M4A (AAC) audio formats
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
- 📋 Playlist management with server-side persistence

## Quick Start

### Prerequisites

- Rust toolchain (Edition 2024)
- A folder with FLAC, MP3, OGG, or M4A music files

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

### Using gtk client

See [music-station-client-gtk](https://github.com/phreer/music-station-client-gtk) for a GTK-based GUI client.

### Using the Web Client

1. Start the server (see above)
2. Open your browser to `http://localhost:3000/web/`
3. Browse tracks, albums, artists, and playlists
4. Click any track to play it; use the queue panel to manage playback order
5. Edit track metadata (title, artist, album, genre, year, etc.) via the edit button
6. Manage lyrics — view, upload, or search online via NetEase / QQ Music

The web client is a Vue 3 single-page application (TypeScript, Pinia, Naive UI) located in `web/`. It is built to `static/` and served at `/web` by the server.

```bash
# Development server with hot reload (proxies API to localhost:3000)
cd web && npm install && npm run dev

# Production build (outputs to ../static/)
cd web && npm run build
```

## API Documentation

Music Station provides a comprehensive REST API for client development.

**📖 For Client Developers:**
- **[CLIENT_DEVELOPMENT_GUIDE.md](CLIENT_DEVELOPMENT_GUIDE.md)** - Quick start guide with examples in JavaScript, Python, Swift, Kotlin
- **[API_DOCUMENTATION.md](API_DOCUMENTATION.md)** - Complete REST API reference with all endpoints, parameters, and responses

## Architecture

The server:

1. Scans the specified library folder on startup
2. Parses audio metadata using Symphonia (FLAC, MP3, OGG, and M4A support)
3. Stores track information in memory (thread-safe with `Arc<RwLock>`)
4. Serves REST API via Axum on port 3000 (configurable)
5. Supports HTTP range requests for efficient audio streaming

The client:

1. Connects to the server via HTTP
2. Fetches and displays track information
3. Provides formatted output for easy browsing
4. Streams and plays audio using Rodio audio library

## License

See LICENSE file for details.
