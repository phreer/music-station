# Client Development Guide

Welcome to the Music Station Client Development Guide! This document helps you get started building GUI clients for the Music Station API.

## Quick Start

### 1. Start the Server

```bash
cargo run -- --library /path/to/your/music
# Server runs on http://localhost:3000
```

### 2. Test the API

```bash
# Verify server is running
curl http://localhost:3000/

# Get all tracks
curl http://localhost:3000/tracks | jq
```

### 3. Build Your Client

Choose your preferred technology and use the API endpoints documented below.

---

## API Overview

Music Station provides a REST API with the following capabilities:

### Core Features
- **Browse Music**: List tracks, albums, and artists
- **Stream Audio**: Play tracks with seeking support (HTTP Range)
- **Manage Metadata**: Update track information (title, artist, album, etc.)
- **Cover Art**: Upload, download, and delete album artwork
- **Lyrics**: Store and retrieve lyrics (plain text or LRC format)
- **Statistics**: Get library overview (counts, sizes, durations)

### Supported Formats
- **Audio**: FLAC (`.flac`) and MP3 (`.mp3`)
- **Images**: JPEG and PNG (for cover art)
- **Lyrics**: Plain text and LRC (timestamped lyrics)

---

## Essential Endpoints

### 📋 Browse Music

```http
GET /tracks              # List all tracks
GET /tracks/:id          # Get track details
GET /albums              # List all albums  
GET /albums/:name        # Get album with tracks
GET /artists             # List all artists
GET /artists/:name       # Get artist with albums
GET /stats               # Library statistics
```

### 🎵 Stream Audio

```http
GET /stream/:id          # Stream track (supports Range requests)
```

**HTML5 Example:**
```html
<audio src="http://localhost:3000/stream/abc123" controls></audio>
```

### ✏️ Update Metadata

```http
PUT /tracks/:id          # Update track metadata
```

**Example:**
```bash
curl -X PUT http://localhost:3000/tracks/abc123 \
  -H "Content-Type: application/json" \
  -d '{"title":"New Title","artist":"New Artist"}'
```

### 🖼️ Cover Art

```http
GET /cover/:id           # Get cover image
POST /cover/:id          # Upload cover (multipart/form-data)
DELETE /cover/:id        # Remove cover
```

### 📝 Lyrics

```http
GET /lyrics/:id          # Get lyrics
PUT /lyrics/:id          # Upload/update lyrics
DELETE /lyrics/:id       # Delete lyrics
```

---

## Data Models

### Track Object

```typescript
{
  id: string,                    // Unique identifier (MD5 of path)
  title: string | null,          // Track title
  artist: string | null,         // Artist name
  album: string | null,          // Album name
  duration_secs: number | null,  // Duration in seconds
  file_size: number,             // File size in bytes
  has_cover: boolean,            // Has embedded cover art
  has_lyrics: boolean,           // Has lyrics in database
  // ... more fields (see API_DOCUMENTATION.md)
}
```

### Album Object

```typescript
{
  name: string,                  // Album name
  artist: string | null,         // Album artist
  track_count: number,           // Number of tracks
  total_duration_secs: number,   // Total duration
  tracks: Track[]                // Array of tracks
}
```

### Artist Object

```typescript
{
  name: string,                  // Artist name
  album_count: number,           // Number of albums
  track_count: number,           // Total tracks
  albums: Album[]                // Array of albums
}
```

---

## Client Examples

### JavaScript/TypeScript (Web)

```javascript
const API_BASE = 'http://localhost:3000';

// Fetch all tracks
async function getTracks() {
  const res = await fetch(`${API_BASE}/tracks`);
  return res.json();
}

// Play a track
function playTrack(trackId) {
  const audio = new Audio(`${API_BASE}/stream/${trackId}`);
  audio.play();
}

// Update track metadata
async function updateTrack(trackId, updates) {
  const res = await fetch(`${API_BASE}/tracks/${trackId}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(updates)
  });
  return res.json();
}

// Display cover art
function showCover(trackId) {
  const img = document.createElement('img');
  img.src = `${API_BASE}/cover/${trackId}`;
  img.onerror = () => img.src = '/placeholder.png';
  return img;
}
```

### Python (requests)

```python
import requests

BASE_URL = 'http://localhost:3000'

def get_tracks():
    """Fetch all tracks"""
    response = requests.get(f'{BASE_URL}/tracks')
    return response.json()

def play_track(track_id):
    """Download and play track"""
    response = requests.get(f'{BASE_URL}/stream/{track_id}', stream=True)
    with open('temp.flac', 'wb') as f:
        for chunk in response.iter_content(chunk_size=8192):
            f.write(chunk)
    # Use your audio library to play temp.flac

def update_metadata(track_id, title=None, artist=None):
    """Update track metadata"""
    updates = {}
    if title: updates['title'] = title
    if artist: updates['artist'] = artist
    
    response = requests.put(
        f'{BASE_URL}/tracks/{track_id}',
        json=updates
    )
    return response.json()
```

### Swift (iOS/macOS)

```swift
import Foundation

class MusicStationAPI {
    let baseURL = "http://localhost:3000"
    
    func getTracks(completion: @escaping ([Track]) -> Void) {
        guard let url = URL(string: "\(baseURL)/tracks") else { return }
        
        URLSession.shared.dataTask(with: url) { data, response, error in
            guard let data = data else { return }
            let tracks = try? JSONDecoder().decode([Track].self, from: data)
            completion(tracks ?? [])
        }.resume()
    }
    
    func streamURL(trackId: String) -> URL? {
        return URL(string: "\(baseURL)/stream/\(trackId)")
    }
    
    func coverURL(trackId: String) -> URL? {
        return URL(string: "\(baseURL)/cover/\(trackId)")
    }
}

// Usage with AVPlayer
import AVFoundation

let player = AVPlayer(url: api.streamURL(trackId: "abc123")!)
player.play()
```

### Kotlin (Android)

```kotlin
import retrofit2.http.*

interface MusicStationAPI {
    @GET("tracks")
    suspend fun getTracks(): List<Track>
    
    @GET("tracks/{id}")
    suspend fun getTrack(@Path("id") id: String): Track
    
    @PUT("tracks/{id}")
    suspend fun updateTrack(
        @Path("id") id: String,
        @Body updates: TrackUpdate
    ): Track
    
    @GET("stream/{id}")
    @Streaming
    suspend fun streamTrack(@Path("id") id: String): ResponseBody
}

// Usage with ExoPlayer
val player = ExoPlayer.Builder(context).build()
val mediaItem = MediaItem.fromUri("http://localhost:3000/stream/$trackId")
player.setMediaItem(mediaItem)
player.prepare()
player.play()
```

### React Component Example

```jsx
import React, { useState, useEffect } from 'react';

function MusicPlayer() {
  const [tracks, setTracks] = useState([]);
  const [currentTrack, setCurrentTrack] = useState(null);

  useEffect(() => {
    // Load tracks on mount
    fetch('http://localhost:3000/tracks')
      .then(res => res.json())
      .then(setTracks);
  }, []);

  return (
    <div className="music-player">
      <h1>My Music Library</h1>
      
      {/* Track List */}
      <ul>
        {tracks.map(track => (
          <li key={track.id}>
            <img 
              src={`http://localhost:3000/cover/${track.id}`}
              alt="Cover"
              width="50"
              onError={e => e.target.style.display = 'none'}
            />
            <span>{track.title || 'Unknown'}</span>
            <button onClick={() => setCurrentTrack(track)}>
              Play
            </button>
          </li>
        ))}
      </ul>

      {/* Audio Player */}
      {currentTrack && (
        <audio
          src={`http://localhost:3000/stream/${currentTrack.id}`}
          controls
          autoPlay
        />
      )}
    </div>
  );
}
```

---

## Best Practices

### 1. Error Handling

Always handle HTTP errors gracefully:

```javascript
async function safeGetTrack(id) {
  try {
    const res = await fetch(`http://localhost:3000/tracks/${id}`);
    if (!res.ok) {
      throw new Error(`HTTP ${res.status}: ${res.statusText}`);
    }
    return await res.json();
  } catch (error) {
    console.error('Failed to fetch track:', error);
    return null;
  }
}
```

### 2. Caching

Cache track lists and metadata to reduce API calls:

```javascript
const cache = new Map();

async function getCachedTracks() {
  if (cache.has('tracks')) {
    return cache.get('tracks');
  }
  const tracks = await fetch('http://localhost:3000/tracks').then(r => r.json());
  cache.set('tracks', tracks);
  return tracks;
}
```

### 3. Cover Art Placeholders

Always provide fallback images for tracks without cover art:

```javascript
<img 
  src={`http://localhost:3000/cover/${track.id}`}
  onError={(e) => e.target.src = '/placeholder.png'}
  alt="Album cover"
/>
```

### 4. Progress Indicators

Show loading states for async operations:

```javascript
const [loading, setLoading] = useState(false);

async function loadTracks() {
  setLoading(true);
  try {
    const tracks = await fetch('http://localhost:3000/tracks').then(r => r.json());
    setTracks(tracks);
  } finally {
    setLoading(false);
  }
}
```

### 5. Range Requests for Seeking

Use Range headers for efficient seeking:

```javascript
// Fetch first 1MB for quick playback start
fetch('http://localhost:3000/stream/abc123', {
  headers: { 'Range': 'bytes=0-1048575' }
})
```

---

## Common Use Cases

### Building a Music Player

**Required Features:**
1. List tracks from `/tracks`
2. Display cover art from `/cover/:id`
3. Stream audio from `/stream/:id` using `<audio>` or native player
4. Show track metadata (title, artist, album, duration)

**Optional Features:**
- Edit metadata via `PUT /tracks/:id`
- Display lyrics from `/lyrics/:id`
- Browse by albums (`/albums`) or artists (`/artists`)
- Show library statistics (`/stats`)

### Building a Metadata Editor

**Required Features:**
1. Load track list from `/tracks`
2. Display editable fields for title, artist, album, etc.
3. Save changes via `PUT /tracks/:id`
4. Upload/manage cover art via `/cover/:id`

### Building a Lyrics Manager

**Required Features:**
1. Load tracks from `/tracks`
2. Fetch existing lyrics from `/lyrics/:id`
3. Edit and save lyrics via `PUT /lyrics/:id`
4. Support both plain text and LRC formats

---

## Testing Your Client

### 1. Start with Sample Data

Create a small test library with 5-10 tracks to develop against.

### 2. Test Edge Cases

- Tracks without metadata (title, artist, album all null)
- Tracks without cover art (`has_cover: false`)
- Tracks without lyrics (`has_lyrics: false`)
- Large files (>100MB) for streaming performance
- Special characters in metadata (Unicode, emojis)

### 3. Use Browser DevTools

Monitor network requests to debug API interactions:
- Open DevTools → Network tab
- Filter by "XHR" or "Fetch"
- Inspect request/response data

### 4. Enable Debug Logging

Start server with verbose logs:

```bash
RUST_LOG=debug cargo run -- --library /path/to/music
```

---

## Reference Implementation

The Music Station includes a built-in web client as a reference:

**URL:** http://localhost:3000/web/index.html

**Source Files:**
- `static/index.html` - HTML structure
- `static/styles.css` - Styling
- `static/app.js` - JavaScript logic

Study these files to see how the API is used in practice.

---

## Troubleshooting

### CORS Errors

The API has CORS enabled for all origins. If you still see CORS errors:
- Ensure you're making requests to `http://localhost:3000` (not `http://127.0.0.1:3000`)
- Check that the server is running
- Verify your client's origin is correct

### Audio Not Playing

- Check browser console for errors
- Verify stream URL: `http://localhost:3000/stream/:id`
- Test stream URL directly in browser address bar
- Ensure audio format is supported (FLAC may need special codecs in some browsers)

### Metadata Not Updating

- Check file permissions (files must be writable)
- Enable debug logging to see detailed error messages
- Verify JSON body format matches `TrackMetadataUpdate` schema
- Check that track ID is correct

### Images Not Loading

- Verify track has cover art (`has_cover: true`)
- Check image URL format: `http://localhost:3000/cover/:id`
- Provide fallback images with `onError` handlers
- Test URL directly in browser

---

## Full API Reference

For complete API documentation, see:
- **[API_DOCUMENTATION.md](./API_DOCUMENTATION.md)** - Full REST API reference with all endpoints, parameters, and examples

---

## Getting Help

1. **Check the logs**: Start server with `RUST_LOG=debug`
2. **Review example code**: See `static/app.js` for working implementation
3. **Test with cURL**: Verify API responses with command-line requests
4. **Check file permissions**: Ensure music files are readable/writable

---

## Next Steps

1. ✅ Start the server: `cargo run -- --library /path/to/music`
2. ✅ Test API with cURL or browser
3. ✅ Choose your client platform/technology
4. ✅ Implement track listing and playback
5. ✅ Add metadata editing features
6. ✅ Implement cover art display
7. ✅ Add lyrics support (optional)

**Happy coding!** 🎵
