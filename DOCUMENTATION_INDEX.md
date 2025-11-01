# Music Station Documentation Index

Complete documentation for the Music Station music server and API.

## 📚 Documentation Overview

### For Client Developers

Building a GUI client? Start here:

1. **[CLIENT_DEVELOPMENT_GUIDE.md](./CLIENT_DEVELOPMENT_GUIDE.md)** (13KB)
   - Quick start guide for GUI client development
   - Code examples in JavaScript, Python, Swift, Kotlin, React
   - Best practices and common use cases
   - Testing and troubleshooting tips
   - **Start here if you're building a desktop, web, or mobile client**

2. **[API_DOCUMENTATION.md](./API_DOCUMENTATION.md)** (20KB)
   - Complete REST API reference
   - All endpoints with parameters and responses
   - Data models and schemas
   - Error handling
   - cURL examples for every endpoint
   - **Comprehensive technical reference**

### For End Users

Using Music Station as a user? Check these guides:

3. **[README.md](./README.md)**
   - Project overview and quick start
   - How to run the server and CLI client
   - Basic usage instructions
   - **Start here if you're new to Music Station**

4. **[CLIENT_USAGE.md](./CLIENT_USAGE.md)** (3.2KB)
   - CLI client command reference
   - Terminal usage examples
   - **For command-line users**

5. **[WEB_CLIENT_GUIDE.md](./WEB_CLIENT_GUIDE.md)** (4.2KB)
   - Web interface features
   - How to browse and manage your library via browser
   - **For web UI users**

### Feature-Specific Guides

Deep dives into specific features:

6. **[LYRICS_GUIDE.md](./LYRICS_GUIDE.md)** (6.5KB)
   - How to use the lyrics feature
   - LRC format support
   - Web UI lyrics display
   - **For users managing lyrics**

7. **[WEB_LYRICS_GUIDE.md](./WEB_LYRICS_GUIDE.md)** (9.0KB)
   - Web client lyrics features in detail
   - Inline editor usage
   - Scrollable display
   - **Detailed web UI lyrics documentation**

8. **[LYRICS_API.md](./LYRICS_API.md)** (11KB)
   - Lyrics fetching API for developers
   - LyricsProvider trait system
   - How to implement custom lyrics providers
   - **For developers building lyrics integrations**

9. **[PLAY_QUEUE_GUIDE.md](./PLAY_QUEUE_GUIDE.md)** (6.3KB)
   - Web player queue management
   - Playlist features
   - **For users of the web player**

10. **[DEBUG_LOGGING.md](./DEBUG_LOGGING.md)**
    - Logging configuration
    - Troubleshooting with logs
    - **For debugging issues**

---

## 🚀 Quick Start by Role

### "I want to build a music player app"
→ Start with [CLIENT_DEVELOPMENT_GUIDE.md](./CLIENT_DEVELOPMENT_GUIDE.md)  
→ Reference [API_DOCUMENTATION.md](./API_DOCUMENTATION.md) as needed

### "I want to use Music Station with my music library"
→ Start with [README.md](./README.md)  
→ Then explore [WEB_CLIENT_GUIDE.md](./WEB_CLIENT_GUIDE.md)

### "I want to understand the lyrics system"
→ Users: [LYRICS_GUIDE.md](./LYRICS_GUIDE.md)  
→ Developers: [LYRICS_API.md](./LYRICS_API.md)

### "I'm having issues and need to debug"
→ Check [DEBUG_LOGGING.md](./DEBUG_LOGGING.md)

---

## 📖 API Quick Reference

### Core Endpoints

```http
GET  /tracks              # List all tracks
GET  /tracks/:id          # Get track details
PUT  /tracks/:id          # Update metadata
GET  /stream/:id          # Stream audio
GET  /cover/:id           # Get cover art
POST /cover/:id           # Upload cover
GET  /lyrics/:id          # Get lyrics
PUT  /lyrics/:id          # Upload lyrics
GET  /playlists           # List playlists
GET  /playlists/:id       # Get playlist
POST /playlists           # Create playlist
PUT  /playlists/:id       # Update playlist
DELETE /playlists/:id     # Delete playlist
GET  /albums              # List albums
GET  /artists             # List artists
GET  /stats               # Library stats
```

**Base URL:** `http://localhost:3000`  
**Format:** JSON  
**CORS:** Enabled for all origins

---

## 🔧 Development Resources

### Source Code Structure

```
src/
├── main.rs           # Server entry point
├── library.rs        # Audio file parsing (FLAC/MP3)
├── lyrics.rs         # Lyrics database
├── server.rs         # REST API handlers
└── bin/
    └── client.rs     # CLI client

static/
├── index.html        # Web client UI
├── app.js            # Web client logic
└── styles.css        # Web client styles
```

### Key Dependencies

- **axum** - Web framework
- **tokio** - Async runtime
- **symphonia** - Audio decoding (FLAC/MP3)
- **metaflac** - FLAC metadata writing
- **id3** - MP3 metadata handling
- **sqlx** - SQLite for lyrics
- **rodio** - Audio playback (CLI)

---

## 🎯 Common Tasks

### Building a Basic Music Player

**Required Reading:**
1. [CLIENT_DEVELOPMENT_GUIDE.md](./CLIENT_DEVELOPMENT_GUIDE.md) - Start here
2. [API_DOCUMENTATION.md](./API_DOCUMENTATION.md) - Sections: Tracks, Streaming, Cover Art

**Key Endpoints:**
- `GET /tracks` - Get track list
- `GET /stream/:id` - Stream audio
- `GET /cover/:id` - Get artwork

### Adding Lyrics Support

**Required Reading:**
1. [LYRICS_GUIDE.md](./LYRICS_GUIDE.md) - User perspective
2. [LYRICS_API.md](./LYRICS_API.md) - Developer API

**Key Endpoints:**
- `GET /lyrics/:id`
- `PUT /lyrics/:id`

### Building a Metadata Editor

**Required Reading:**
1. [API_DOCUMENTATION.md](./API_DOCUMENTATION.md) - Section: Update Metadata
2. [CLIENT_DEVELOPMENT_GUIDE.md](./CLIENT_DEVELOPMENT_GUIDE.md) - Examples

**Key Endpoints:**
- `GET /tracks/:id`
- `PUT /tracks/:id`
- `POST /cover/:id`

### Managing Playlists

**Required Reading:**
1. [API_DOCUMENTATION.md](./API_DOCUMENTATION.md) - Section: Playlists

**Key Endpoints:**
- `GET /playlists` - List all playlists
- `GET /playlists/:id` - Get playlist details
- `POST /playlists` - Create new playlist
- `PUT /playlists/:id` - Update playlist (name, description, tracks)
- `DELETE /playlists/:id` - Delete playlist

---

## 💡 Tips for Client Developers

### 1. Start Simple
Build track listing and playback first, then add features incrementally.

### 2. Handle Missing Data
Not all tracks have metadata, cover art, or lyrics. Always provide fallbacks.

### 3. Use HTTP Range Requests
For seeking in audio players, use Range headers with `/stream/:id`

### 4. Cache Wisely
Cache track lists and metadata to reduce API calls, but refresh periodically.

### 5. Test with Real Data
Use a diverse library with missing metadata, Unicode characters, and large files.

### 6. Enable Debug Logging
Start server with `RUST_LOG=debug` to troubleshoot API issues.

---

## 🐛 Troubleshooting

### API Returns 404
- Check track ID is correct (MD5 hash of file path)
- Verify server is running and library is scanned
- Test with `curl http://localhost:3000/tracks`

### Audio Won't Play
- Check browser console for errors
- Verify stream URL format: `/stream/:id`
- Test URL directly in browser address bar
- Some browsers need specific codecs for FLAC

### Metadata Updates Fail
- Check file permissions (must be writable)
- Enable debug logging for detailed errors
- Verify JSON format matches schema

### CORS Errors
- Server has CORS enabled by default
- Ensure correct base URL (not mixing localhost/127.0.0.1)

See [DEBUG_LOGGING.md](./DEBUG_LOGGING.md) for more troubleshooting.

---

## 📝 Contributing

When adding new features:

1. Update relevant API documentation in `API_DOCUMENTATION.md`
2. Add examples to `CLIENT_DEVELOPMENT_GUIDE.md`
3. Update feature-specific guides as needed
4. Add entry to this index

---

## 📄 License

See [LICENSE](./LICENSE) for details.

---

**Last Updated:** 2025-10-23  
**Server Version:** 0.1.0
