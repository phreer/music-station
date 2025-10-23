# Music Station REST API Documentation

**Version:** 0.1.0  
**Base URL:** `http://localhost:3000`  
**Protocol:** HTTP/REST  
**Response Format:** JSON

## Table of Contents
- [Overview](#overview)
- [Getting Started](#getting-started)
- [Authentication](#authentication)
- [Data Models](#data-models)
- [API Endpoints](#api-endpoints)
  - [General](#general)
  - [Tracks](#tracks)
  - [Albums](#albums)
  - [Artists](#artists)
  - [Cover Art](#cover-art)
  - [Lyrics](#lyrics)
  - [Statistics](#statistics)
- [Error Handling](#error-handling)
- [Client Development Examples](#client-development-examples)

---

## Overview

Music Station provides a RESTful API for managing and streaming a music library. The server scans a local folder containing audio files (FLAC and MP3) and exposes metadata, streaming, and management capabilities through HTTP endpoints.

**Key Features:**
- Browse tracks, albums, and artists
- Stream audio with HTTP Range support (seeking)
- Update track metadata (tags)
- Manage embedded cover art
- Store and retrieve lyrics
- Get library statistics

**Supported Audio Formats:**
- FLAC (`.flac`)
- MP3 (`.mp3`)

---

## Getting Started

### Starting the Server

```bash
# Start server with library path
cargo run -- --library /path/to/music/folder

# Or use environment variable
MUSIC_LIBRARY_PATH=/path/to/music cargo run

# Server starts on http://localhost:3000 by default
```

### Making Your First Request

```bash
# Check server status
curl http://localhost:3000/

# List all tracks
curl http://localhost:3000/tracks
```

---

## Authentication

Currently, the API does not require authentication. All endpoints are publicly accessible.

---

## Data Models

### Track

```typescript
{
  id: string,                         // MD5 hash of file path
  path: string,                       // Absolute file path
  title: string | null,               // Track title
  artist: string | null,              // Track artist
  album: string | null,               // Album name
  album_artist: string | null,        // Album artist
  genre: string | null,               // Genre
  year: string | null,                // Release year
  track_number: string | null,        // Track number
  disc_number: string | null,         // Disc number
  composer: string | null,            // Composer
  comment: string | null,             // Comment
  duration_secs: number | null,       // Duration in seconds
  file_size: number,                  // File size in bytes
  has_cover: boolean,                 // Has embedded cover art
  has_lyrics: boolean,                // Has lyrics in database
  custom_fields: Record<string, string> // Other metadata tags
}
```

### Album

```typescript
{
  name: string,                       // Album name
  artist: string | null,              // Album artist
  track_count: number,                // Number of tracks
  total_duration_secs: number,        // Total duration in seconds
  tracks: Track[]                     // Array of tracks
}
```

### Artist

```typescript
{
  name: string,                       // Artist name
  album_count: number,                // Number of albums
  track_count: number,                // Total number of tracks
  albums: Album[]                     // Array of albums
}
```

### LibraryStats

```typescript
{
  total_tracks: number,               // Total number of tracks
  total_albums: number,               // Total number of albums
  total_artists: number,              // Total number of artists
  total_duration_secs: number,        // Total duration in seconds
  total_size_bytes: number            // Total file size in bytes
}
```

### Lyric

```typescript
{
  track_id: string,                   // Track ID
  content: string,                    // Lyrics content
  format: "plain" | "lrc",            // Format type
  language: string | null,            // Language code (e.g., "en", "zh")
  source: string | null,              // Source (e.g., "manual", "genius")
  created_at: string,                 // ISO 8601 timestamp
  updated_at: string                  // ISO 8601 timestamp
}
```

### TrackMetadataUpdate

```typescript
{
  title?: string,
  artist?: string,
  album?: string,
  album_artist?: string,
  genre?: string,
  year?: string,
  track_number?: string,
  disc_number?: string,
  composer?: string,
  comment?: string,
  custom_fields?: Record<string, string>
}
```

### LyricUpload

```typescript
{
  content: string,                    // Lyrics content (required)
  format?: "plain" | "lrc",           // Format (auto-detected if omitted)
  language?: string,                  // Language code
  source?: string                     // Source name
}
```

---

## API Endpoints

### General

#### Get API Version

```http
GET /
```

**Response:**
```
200 OK
Content-Type: text/plain

Music Station API v0.1.0
```

---

### Tracks

#### List All Tracks

```http
GET /tracks
```

**Response:**
```json
200 OK
Content-Type: application/json

[
  {
    "id": "a1b2c3d4...",
    "path": "/music/song.flac",
    "title": "Example Song",
    "artist": "Example Artist",
    "album": "Example Album",
    "duration_secs": 240,
    "file_size": 30000000,
    "has_cover": true,
    "has_lyrics": false,
    ...
  }
]
```

#### Get Track by ID

```http
GET /tracks/:id
```

**Parameters:**
- `id` (path) - Track ID

**Response:**
```json
200 OK
Content-Type: application/json

{
  "id": "a1b2c3d4...",
  "title": "Example Song",
  ...
}
```

**Errors:**
- `404 Not Found` - Track not found

#### Update Track Metadata

```http
PUT /tracks/:id
Content-Type: application/json
```

**Parameters:**
- `id` (path) - Track ID

**Request Body:**
```json
{
  "title": "New Title",
  "artist": "New Artist",
  "album": "New Album"
}
```

All fields are optional. Only provided fields will be updated.

**Response:**
```json
200 OK
Content-Type: application/json

{
  "id": "a1b2c3d4...",
  "title": "New Title",
  "artist": "New Artist",
  ...
}
```

**Errors:**
- `404 Not Found` - Track not found
- `500 Internal Server Error` - Failed to write metadata (e.g., read-only file)

**Notes:**
- Updates the audio file's embedded metadata tags
- FLAC uses Vorbis comments (TITLE, ARTIST, ALBUM, etc.)
- MP3 uses ID3v2 tags (TIT2, TPE1, TALB, etc.)
- The `has_lyrics` flag is preserved during updates

#### Stream Track

```http
GET /stream/:id
```

**Parameters:**
- `id` (path) - Track ID

**Headers:**
- `Range` (optional) - Byte range (e.g., `bytes=0-1023`, `bytes=1024-`, `bytes=-500`)

**Response (Full File):**
```http
200 OK
Content-Type: audio/flac    (or audio/mpeg for MP3)
Content-Length: 30000000
Accept-Ranges: bytes
Content-Disposition: inline; filename="song.flac"

<binary audio data>
```

**Response (Range Request):**
```http
206 Partial Content
Content-Type: audio/flac
Content-Length: 1024
Content-Range: bytes 0-1023/30000000
Accept-Ranges: bytes

<binary audio data>
```

**Errors:**
- `404 Not Found` - Track not found

**Notes:**
- Supports HTTP Range requests for seeking/streaming
- Used by HTML5 `<audio>` elements for progressive loading
- Range formats:
  - `bytes=start-end` - Specific range
  - `bytes=start-` - From start to end of file
  - `bytes=-N` - Last N bytes

---

### Albums

#### List All Albums

```http
GET /albums
```

**Response:**
```json
200 OK
Content-Type: application/json

[
  {
    "name": "Example Album",
    "artist": "Example Artist",
    "track_count": 12,
    "total_duration_secs": 2880,
    "tracks": [...]
  }
]
```

**Notes:**
- Tracks are grouped by the `album` field
- Albums are sorted alphabetically by name

#### Get Album by Name

```http
GET /albums/:name
```

**Parameters:**
- `name` (path) - Album name (URL-encoded)

**Example:**
```http
GET /albums/Dark%20Side%20of%20the%20Moon
```

**Response:**
```json
200 OK
Content-Type: application/json

{
  "name": "Dark Side of the Moon",
  "artist": "Pink Floyd",
  "track_count": 10,
  "total_duration_secs": 2580,
  "tracks": [...]
}
```

**Errors:**
- `404 Not Found` - Album not found

---

### Artists

#### List All Artists

```http
GET /artists
```

**Response:**
```json
200 OK
Content-Type: application/json

[
  {
    "name": "Example Artist",
    "album_count": 5,
    "track_count": 60,
    "albums": [...]
  }
]
```

**Notes:**
- Artists are grouped by the `artist` field (not `album_artist`)
- Artists are sorted alphabetically by name

#### Get Artist by Name

```http
GET /artists/:name
```

**Parameters:**
- `name` (path) - Artist name (URL-encoded)

**Example:**
```http
GET /artists/Pink%20Floyd
```

**Response:**
```json
200 OK
Content-Type: application/json

{
  "name": "Pink Floyd",
  "album_count": 15,
  "track_count": 150,
  "albums": [...]
}
```

**Errors:**
- `404 Not Found` - Artist not found

---

### Cover Art

#### Get Cover Art

```http
GET /cover/:id
```

**Parameters:**
- `id` (path) - Track ID

**Response:**
```http
200 OK
Content-Type: image/jpeg    (or image/png)
Cache-Control: public, max-age=3600

<binary image data>
```

**Errors:**
- `404 Not Found` - Track not found or no cover art
- `500 Internal Server Error` - Failed to read cover art

**Notes:**
- Returns embedded cover art from audio file
- MIME type auto-detected from image data
- Cached for 1 hour

#### Upload Cover Art

```http
POST /cover/:id
Content-Type: multipart/form-data
```

**Parameters:**
- `id` (path) - Track ID

**Request Body (Multipart Form):**
```
--boundary
Content-Disposition: form-data; name="image"; filename="cover.jpg"
Content-Type: image/jpeg

<binary image data>
--boundary--
```

**Form Field Names:**
- `image` or `cover` - Image file

**Response:**
```json
200 OK
Content-Type: application/json

{
  "id": "a1b2c3d4...",
  "has_cover": true,
  ...
}
```

**Errors:**
- `400 Bad Request` - No image data or invalid multipart
- `404 Not Found` - Track not found
- `500 Internal Server Error` - Failed to write cover art

**Notes:**
- Embeds image into audio file metadata
- FLAC: Stored as PICTURE block
- MP3: Stored as APIC frame (ID3v2)
- Replaces existing cover art

#### Delete Cover Art

```http
DELETE /cover/:id
```

**Parameters:**
- `id` (path) - Track ID

**Response:**
```json
200 OK
Content-Type: application/json

{
  "id": "a1b2c3d4...",
  "has_cover": false,
  ...
}
```

**Errors:**
- `404 Not Found` - Track not found
- `500 Internal Server Error` - Failed to remove cover art

---

### Lyrics

#### Get Lyrics

```http
GET /lyrics/:id
```

**Parameters:**
- `id` (path) - Track ID

**Response:**
```json
200 OK
Content-Type: application/json

{
  "track_id": "a1b2c3d4...",
  "content": "Verse 1\nLyrics here...",
  "format": "plain",
  "language": "en",
  "source": "manual",
  "created_at": "2024-01-01T12:00:00Z",
  "updated_at": "2024-01-01T12:00:00Z"
}
```

**Errors:**
- `404 Not Found` - Track not found or no lyrics
- `500 Internal Server Error` - Database error

**Notes:**
- Lyrics are stored in a separate SQLite database
- Not embedded in audio files

#### Upload/Update Lyrics

```http
PUT /lyrics/:id
Content-Type: application/json
```

**Parameters:**
- `id` (path) - Track ID

**Request Body:**
```json
{
  "content": "Verse 1\nLyrics here...",
  "format": "plain",
  "language": "en",
  "source": "genius"
}
```

**Fields:**
- `content` (required) - Lyrics text
- `format` (optional) - "plain" or "lrc" (auto-detected if omitted)
- `language` (optional) - Language code (e.g., "en", "zh")
- `source` (optional) - Source name (e.g., "manual", "genius")

**Response:**
```json
200 OK
Content-Type: application/json

{
  "track_id": "a1b2c3d4...",
  "content": "Verse 1\nLyrics here...",
  "format": "plain",
  "language": "en",
  "source": "genius",
  "created_at": "2024-01-01T12:00:00Z",
  "updated_at": "2024-01-01T12:00:00Z"
}
```

**Errors:**
- `404 Not Found` - Track not found
- `500 Internal Server Error` - Failed to save lyrics

**Notes:**
- Creates new lyrics or updates existing ones
- Updates track's `has_lyrics` flag
- LRC format auto-detected if content contains timestamps like `[00:12.34]`

#### Delete Lyrics

```http
DELETE /lyrics/:id
```

**Parameters:**
- `id` (path) - Track ID

**Response:**
```http
204 No Content
```

**Errors:**
- `404 Not Found` - Track not found or no lyrics
- `500 Internal Server Error` - Failed to delete lyrics

**Notes:**
- Removes lyrics from database
- Updates track's `has_lyrics` flag to false

---

### Statistics

#### Get Library Statistics

```http
GET /stats
```

**Response:**
```json
200 OK
Content-Type: application/json

{
  "total_tracks": 1234,
  "total_albums": 120,
  "total_artists": 45,
  "total_duration_secs": 345600,
  "total_size_bytes": 12000000000
}
```

---

## Error Handling

### HTTP Status Codes

| Code | Meaning |
|------|---------|
| 200 | OK - Request successful |
| 204 | No Content - Successful deletion |
| 206 | Partial Content - Range request successful |
| 400 | Bad Request - Invalid request data |
| 404 | Not Found - Resource not found |
| 500 | Internal Server Error - Server-side error |

### Error Response Format

Most errors return only an HTTP status code. For detailed error messages, check server logs.

```bash
# Enable verbose logging
RUST_LOG=debug cargo run -- --library /path/to/music
```

---

## Client Development Examples

### JavaScript (Fetch API)

```javascript
// List all tracks
async function getTracks() {
  const response = await fetch('http://localhost:3000/tracks');
  const tracks = await response.json();
  return tracks;
}

// Get track details
async function getTrack(id) {
  const response = await fetch(`http://localhost:3000/tracks/${id}`);
  if (!response.ok) throw new Error('Track not found');
  return await response.json();
}

// Update track metadata
async function updateTrack(id, updates) {
  const response = await fetch(`http://localhost:3000/tracks/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(updates)
  });
  return await response.json();
}

// Stream audio (HTML5 audio element)
function playTrack(id) {
  const audio = new Audio(`http://localhost:3000/stream/${id}`);
  audio.play();
  return audio;
}

// Upload cover art
async function uploadCover(id, imageFile) {
  const formData = new FormData();
  formData.append('image', imageFile);
  
  const response = await fetch(`http://localhost:3000/cover/${id}`, {
    method: 'POST',
    body: formData
  });
  return await response.json();
}

// Upload lyrics
async function uploadLyrics(id, content, format = 'plain') {
  const response = await fetch(`http://localhost:3000/lyrics/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ content, format })
  });
  return await response.json();
}

// Get library stats
async function getStats() {
  const response = await fetch('http://localhost:3000/stats');
  return await response.json();
}
```

### Python (requests)

```python
import requests

BASE_URL = 'http://localhost:3000'

# List all tracks
def get_tracks():
    response = requests.get(f'{BASE_URL}/tracks')
    return response.json()

# Get track details
def get_track(track_id):
    response = requests.get(f'{BASE_URL}/tracks/{track_id}')
    response.raise_for_status()
    return response.json()

# Update track metadata
def update_track(track_id, updates):
    response = requests.put(
        f'{BASE_URL}/tracks/{track_id}',
        json=updates
    )
    return response.json()

# Download audio stream
def download_track(track_id, output_path):
    response = requests.get(
        f'{BASE_URL}/stream/{track_id}',
        stream=True
    )
    with open(output_path, 'wb') as f:
        for chunk in response.iter_content(chunk_size=8192):
            f.write(chunk)

# Upload cover art
def upload_cover(track_id, image_path):
    with open(image_path, 'rb') as f:
        files = {'image': f}
        response = requests.post(
            f'{BASE_URL}/cover/{track_id}',
            files=files
        )
    return response.json()

# Upload lyrics
def upload_lyrics(track_id, content, format='plain'):
    response = requests.put(
        f'{BASE_URL}/lyrics/{track_id}',
        json={'content': content, 'format': format}
    )
    return response.json()

# Get library statistics
def get_stats():
    response = requests.get(f'{BASE_URL}/stats')
    return response.json()
```

### cURL Examples

```bash
# List tracks
curl http://localhost:3000/tracks

# Get specific track
curl http://localhost:3000/tracks/abc123

# Update track metadata
curl -X PUT http://localhost:3000/tracks/abc123 \
  -H "Content-Type: application/json" \
  -d '{"title":"New Title","artist":"New Artist"}'

# Stream track (save to file)
curl http://localhost:3000/stream/abc123 -o song.flac

# Stream with range (first 1MB)
curl http://localhost:3000/stream/abc123 \
  -H "Range: bytes=0-1048575" -o chunk.flac

# Get cover art
curl http://localhost:3000/cover/abc123 -o cover.jpg

# Upload cover art
curl -X POST http://localhost:3000/cover/abc123 \
  -F "image=@cover.jpg"

# Delete cover art
curl -X DELETE http://localhost:3000/cover/abc123

# Get lyrics
curl http://localhost:3000/lyrics/abc123

# Upload lyrics
curl -X PUT http://localhost:3000/lyrics/abc123 \
  -H "Content-Type: application/json" \
  -d '{"content":"Verse 1\nLyrics...","format":"plain"}'

# Delete lyrics
curl -X DELETE http://localhost:3000/lyrics/abc123

# Get statistics
curl http://localhost:3000/stats

# List albums
curl http://localhost:3000/albums

# Get specific album (URL encode spaces)
curl "http://localhost:3000/albums/Dark%20Side%20of%20the%20Moon"

# List artists
curl http://localhost:3000/artists
```

### React Example Component

```jsx
import React, { useState, useEffect } from 'react';

function TrackList() {
  const [tracks, setTracks] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch('http://localhost:3000/tracks')
      .then(res => res.json())
      .then(data => {
        setTracks(data);
        setLoading(false);
      })
      .catch(err => console.error(err));
  }, []);

  const playTrack = (id) => {
    const audio = new Audio(`http://localhost:3000/stream/${id}`);
    audio.play();
  };

  if (loading) return <div>Loading...</div>;

  return (
    <div>
      <h1>Tracks ({tracks.length})</h1>
      <ul>
        {tracks.map(track => (
          <li key={track.id}>
            <img 
              src={`http://localhost:3000/cover/${track.id}`}
              alt="Cover"
              width="50"
              onError={(e) => e.target.style.display = 'none'}
            />
            <strong>{track.title || 'Unknown'}</strong>
            <span> - {track.artist || 'Unknown Artist'}</span>
            <button onClick={() => playTrack(track.id)}>Play</button>
          </li>
        ))}
      </ul>
    </div>
  );
}
```

---

## Additional Notes

### CORS

The API has CORS enabled for all origins (`CorsLayer::permissive()`). This allows web clients from any domain to access the API.

### File Permissions

When updating metadata or cover art on MP3 files, ensure files are writable. Read-only files will return `500 Internal Server Error`.

```bash
# Make file writable
chmod u+w /path/to/music/song.mp3
```

### Track ID Stability

**Important:** Track IDs are MD5 hashes of file paths. If a file is moved or renamed, its ID will change. This affects:
- Lyrics database links (lyrics may become orphaned)
- Client bookmarks/playlists

Consider using inode numbers or embedding UUIDs in metadata for stable IDs.

### Database Location

Lyrics are stored in `./lyrics.db` in the server's working directory. Back up this file to preserve lyrics data.

### Web Client

The server includes a built-in web client at:
```
http://localhost:3000/web/index.html
```

This serves as a reference implementation using the API.

---

## Support

For issues or questions:
- Check server logs: `RUST_LOG=debug cargo run -- --library /path/to/music`
- Review source code: `src/server.rs`
- Examine data structures: `src/library.rs`

**Server Version:** 0.1.0  
**Documentation Last Updated:** 2025-10-23
