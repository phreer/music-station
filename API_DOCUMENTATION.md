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
  - [Playlists](#playlists)
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
  play_count: number,                 // Number of times played
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
  format: "plain" | "lrc" | "lrc_word", // Format type
  language: string | null,            // Language code (e.g., "en", "zh")
  source: string | null,              // Source (e.g., "manual", "genius", "netease", "qqmusic")
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
  format?: "plain" | "lrc" | "lrc_word", // Format (auto-detected if omitted)
  language?: string,                  // Language code
  source?: string                     // Source name
}
```

### Playlist

```typescript
{
  id: string,                         // UUID
  name: string,                       // Playlist name
  description: string | null,         // Optional description
  track_ids: string[],                // Array of track IDs
  created_at: string,                 // ISO 8601 timestamp
  updated_at: string                  // ISO 8601 timestamp
}
```

### PlaylistCreate

```typescript
{
  name: string,                       // Playlist name (required)
  description?: string,               // Optional description
  track_ids?: string[]                // Initial tracks (default: empty array)
}
```

### PlaylistUpdate

```typescript
{
  name?: string,                      // New playlist name
  description?: string | null,        // New description (null to clear)
  track_ids?: string[]                // New track list (replaces entire list)
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

#### Increment Play Count

```http
POST /tracks/:id/play
```

Increments the play count for the specified track and updates the last played timestamp.

**Parameters:**
- `id` (path) - Track ID

**Response:**
```json
200 OK
Content-Type: application/json

5
```
Returns the new play count as a number.

**Errors:**
- `404 Not Found` - Track not found
- `500 Internal Server Error` - Database error

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

#### Search Lyrics

```http
GET /lyrics/search?q={query}&provider={provider}&artist={artist}
```

**Query Parameters:**
- `q` (required) - Search query (usually track title)
- `provider` (required) - Lyrics provider: "netease" or "qqmusic"
- `artist` (optional) - Artist name for better matching

**Example:**
```http
GET /lyrics/search?q=Norwegian%20Wood&provider=qqmusic&artist=The%20Beatles
```

**Response:**
```json
200 OK
Content-Type: application/json

[
  {
    "id": "12345",
    "title": "Norwegian Wood",
    "artist": "The Beatles",
    "album": "Rubber Soul",
    "duration": 125000,
    "confidence": 0.95
  }
]
```

**Errors:**
- `400 Bad Request` - Invalid provider or missing required parameters
- `500 Internal Server Error` - Provider search failed
- `503 Service Unavailable` - Provider not initialized

**Notes:**
- Returns search results ranked by confidence (0.0 to 1.0)
- NetEase Cloud Music (网易云音乐) and QQ Music (QQ音乐) providers supported
- Results include song metadata for verification before fetching

#### Fetch Lyrics from Provider

```http
GET /lyrics/fetch/:provider/:song_id
```

**Parameters:**
- `provider` (path) - Lyrics provider: "netease" or "qqmusic"
- `song_id` (path) - Song ID from search results

**Example:**
```http
GET /lyrics/fetch/qqmusic/12345
```

**Response:**
```json
200 OK
Content-Type: application/json

{
  "content": "[0,11550]挪(0,721)威(721,721)的(1442,721)森(2163,721)林(2884,721)\n[11550,5000]Another(0,500) line(500,500)",
  "format": "lrc_word",
  "language": "zh",
  "source": "qqmusic",
  "url": "https://y.qq.com/n/ryqq/songDetail/12345",
  "metadata": {
    "copyright": "QQ Music"
  }
}
```

**Errors:**
- `400 Bad Request` - Invalid provider
- `404 Not Found` - Song ID not found
- `500 Internal Server Error` - Failed to fetch lyrics
- `503 Service Unavailable` - Provider not initialized

**Notes:**
- Returns lyrics with auto-detected format (`plain`, `lrc`, or `lrc_word`)
- QQ Music often provides word-level synchronized lyrics (`lrc_word`)
- NetEase Cloud Music typically provides line-level synchronized lyrics (`lrc`)
- Includes source URL and metadata
- This does NOT save lyrics to database - use `PUT /lyrics/:id` to save

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
- `format` (optional) - "plain", "lrc", or "lrc_word" (auto-detected if omitted)
- `language` (optional) - Language code (e.g., "en", "zh")
- `source` (optional) - Source name (e.g., "manual", "genius", "netease", "qqmusic")

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
- Format is auto-detected from content:
  - **`lrc_word`**: Detected if content contains word-level timing like `word(offset,duration)`
  - **`lrc`**: Detected if content contains line timestamps like `[00:12.34]` or `[offset,duration]`
  - **`plain`**: Default for plain text without timing data
- Word-level LRC format example: `[0,11550]挪(0,721)威(721,721)的(1442,721)森(2163,721)林(2884,721)`

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

### Playlists

#### List All Playlists

```http
GET /playlists
```

**Response:**
```json
200 OK
Content-Type: application/json

[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "My Favorites",
    "description": "My favorite tracks",
    "track_ids": ["a1b2c3d4...", "e5f6g7h8..."],
    "created_at": "2024-01-01T12:00:00Z",
    "updated_at": "2024-01-01T12:00:00Z"
  }
]
```

**Notes:**
- Returns all playlists sorted by creation date (newest first)

#### Get Playlist by ID

```http
GET /playlists/:id
```

**Parameters:**
- `id` (path) - Playlist ID (UUID)

**Response:**
```json
200 OK
Content-Type: application/json

{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "My Favorites",
  "description": "My favorite tracks",
  "track_ids": ["a1b2c3d4...", "e5f6g7h8..."],
  "created_at": "2024-01-01T12:00:00Z",
  "updated_at": "2024-01-01T12:00:00Z"
}
```

**Errors:**
- `404 Not Found` - Playlist not found

#### Create Playlist

```http
POST /playlists
Content-Type: application/json
```

**Request Body:**
```json
{
  "name": "My Favorites",
  "description": "My favorite tracks",
  "track_ids": ["a1b2c3d4...", "e5f6g7h8..."]
}
```

**Fields:**
- `name` (required) - Playlist name (must be unique)
- `description` (optional) - Playlist description
- `track_ids` (optional) - Initial track IDs (default: empty array)

**Response:**
```json
201 Created
Content-Type: application/json

{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "My Favorites",
  "description": "My favorite tracks",
  "track_ids": ["a1b2c3d4...", "e5f6g7h8..."],
  "created_at": "2024-01-01T12:00:00Z",
  "updated_at": "2024-01-01T12:00:00Z"
}
```

**Errors:**
- `400 Bad Request` - Invalid request body (missing name or duplicate name)
- `500 Internal Server Error` - Failed to create playlist

**Notes:**
- Playlist names must be unique across all playlists
- Returns 400 Bad Request if a playlist with the same name already exists

#### Update Playlist

```http
PUT /playlists/:id
Content-Type: application/json
```

**Parameters:**
- `id` (path) - Playlist ID (UUID)

**Request Body:**
```json
{
  "name": "Updated Favorites",
  "description": "My updated favorite tracks",
  "track_ids": ["a1b2c3d4...", "e5f6g7h8...", "i9j0k1l2..."]
}
```

**Fields:**
- `name` (optional) - New playlist name (must be unique)
- `description` (optional) - New description (use `null` to clear)
- `track_ids` (optional) - New track list (replaces entire list)

**Response:**
```json
200 OK
Content-Type: application/json

{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Updated Favorites",
  "description": "My updated favorite tracks",
  "track_ids": ["a1b2c3d4...", "e5f6g7h8...", "i9j0k1l2..."],
  "created_at": "2024-01-01T12:00:00Z",
  "updated_at": "2024-01-02T15:30:00Z"
}
```

**Errors:**
- `400 Bad Request` - Duplicate playlist name
- `404 Not Found` - Playlist not found
- `500 Internal Server Error` - Failed to update playlist

**Notes:**
- All fields are optional - only provided fields will be updated
- Playlist names must be unique; returns 400 Bad Request if name is already taken
- Track IDs are stored as a comma-separated list in the database
- Invalid track IDs are not validated; they are stored as-is

#### Delete Playlist

```http
DELETE /playlists/:id
```

**Parameters:**
- `id` (path) - Playlist ID (UUID)

**Response:**
```http
204 No Content
```

**Errors:**
- `404 Not Found` - Playlist not found
- `500 Internal Server Error` - Failed to delete playlist

**Notes:**
- Permanently deletes the playlist
- Does not affect the tracks themselves

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
  "total_size_bytes": 12000000000,
  "total_plays": 5678
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

// Search lyrics
async function searchLyrics(query, provider, artist = null) {
  const params = new URLSearchParams({ q: query, provider });
  if (artist) params.append('artist', artist);
  
  const response = await fetch(`http://localhost:3000/lyrics/search?${params}`);
  return await response.json();
}

// Fetch lyrics from provider
async function fetchLyricsFromProvider(provider, songId) {
  const response = await fetch(
    `http://localhost:3000/lyrics/fetch/${provider}/${songId}`
  );
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

// List all playlists
async function getPlaylists() {
  const response = await fetch('http://localhost:3000/playlists');
  return await response.json();
}

// Get playlist by ID
async function getPlaylist(id) {
  const response = await fetch(`http://localhost:3000/playlists/${id}`);
  if (!response.ok) throw new Error('Playlist not found');
  return await response.json();
}

// Create playlist
async function createPlaylist(name, description = null, trackIds = []) {
  const response = await fetch('http://localhost:3000/playlists', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, description, track_ids: trackIds })
  });
  return await response.json();
}

// Update playlist
async function updatePlaylist(id, updates) {
  const response = await fetch(`http://localhost:3000/playlists/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(updates)
  });
  return await response.json();
}

// Delete playlist
async function deletePlaylist(id) {
  const response = await fetch(`http://localhost:3000/playlists/${id}`, {
    method: 'DELETE'
  });
  if (!response.ok) throw new Error('Failed to delete playlist');
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

# List all playlists
def get_playlists():
    response = requests.get(f'{BASE_URL}/playlists')
    return response.json()

# Get playlist by ID
def get_playlist(playlist_id):
    response = requests.get(f'{BASE_URL}/playlists/{playlist_id}')
    response.raise_for_status()
    return response.json()

# Create playlist
def create_playlist(name, description=None, track_ids=None):
    data = {'name': name}
    if description:
        data['description'] = description
    if track_ids:
        data['track_ids'] = track_ids
    
    response = requests.post(
        f'{BASE_URL}/playlists',
        json=data
    )
    return response.json()

# Update playlist
def update_playlist(playlist_id, updates):
    response = requests.put(
        f'{BASE_URL}/playlists/{playlist_id}',
        json=updates
    )
    return response.json()

# Delete playlist
def delete_playlist(playlist_id):
    response = requests.delete(f'{BASE_URL}/playlists/{playlist_id}')
    response.raise_for_status()
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

# Search lyrics
curl "http://localhost:3000/lyrics/search?q=Norwegian%20Wood&provider=qqmusic&artist=The%20Beatles"

# Fetch lyrics from provider
curl http://localhost:3000/lyrics/fetch/qqmusic/12345

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

### Lyrics Formats

The API supports three lyrics formats with automatic detection:

**Plain Text (`plain`)**
```
Verse 1
This is plain text lyrics
No timing information
```

**Line-Level LRC (`lrc`)**
```
[00:12.34]This is a line of lyrics
[00:16.78]Another line follows
[00:20.12]Standard LRC format
```

**Word-Level LRC (`lrc_word`)**
```
[0,11550]挪(0,721)威(721,721)的(1442,721)森(2163,721)林(2884,721)
[11550,5000]Another(0,500) line(500,500) with(1000,300) words(1300,400)
```

Format is automatically detected based on content:
- Contains `word(offset,duration)` pattern → `lrc_word`
- Contains `[mm:ss.xx]` or `[offset,duration]` → `lrc`
- Otherwise → `plain`

Word-level lyrics enable karaoke-style word-by-word highlighting in the web client.

### Lyrics Providers

Two lyrics providers are integrated:

**NetEase Cloud Music (网易云音乐)**
- Provider ID: `netease`
- Typically provides line-level LRC lyrics
- Large Chinese music library
- Some international tracks available

**QQ Music (QQ音乐)**
- Provider ID: `qqmusic`
- Often provides word-level LRC lyrics with karaoke timing
- Extensive Chinese music library
- High-quality synchronized lyrics

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
**Documentation Last Updated:** 2025-10-26
