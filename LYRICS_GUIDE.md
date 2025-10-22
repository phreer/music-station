# Lyrics Feature Guide

## Overview

Music Station now supports lyrics storage and management for your music library. Lyrics are stored in a SQLite database and can be uploaded, retrieved, and deleted through REST API endpoints.

## Features

- **Persistent Storage**: Lyrics are stored in a SQLite database at `.music-station/lyrics.db` in your library folder
- **Multiple Formats**: Supports both plain text and LRC (synchronized lyrics) formats
- **Metadata Support**: Store language, source, and timestamps with lyrics
- **Track Integration**: The `has_lyrics` flag is automatically updated on tracks
- **Auto-format Detection**: Automatically detects LRC format based on content

## Database Schema

The lyrics database contains a single table:

```sql
CREATE TABLE lyrics (
    track_id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    format TEXT NOT NULL,       -- 'plain' or 'lrc'
    language TEXT,               -- Optional: e.g., 'en', 'es', 'ja'
    source TEXT,                 -- Optional: source/origin of lyrics
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

## API Endpoints

### Get Lyrics for a Track

```http
GET /lyrics/:id
```

**Response** (200 OK):
```json
{
  "track_id": "abc123",
  "content": "[00:12.00]First line of lyrics\n[00:16.50]Second line",
  "format": "lrc",
  "language": "en",
  "source": "Manual upload",
  "created_at": "2025-10-22T12:00:00Z",
  "updated_at": "2025-10-22T12:00:00Z"
}
```

**Error Responses**:
- `404 Not Found`: Track doesn't exist or has no lyrics

### Upload/Update Lyrics

```http
PUT /lyrics/:id
Content-Type: application/json
```

**Request Body**:
```json
{
  "content": "[00:12.00]First line\n[00:16.50]Second line",
  "format": "lrc",       // Optional: "plain" or "lrc" (auto-detected if omitted)
  "language": "en",      // Optional
  "source": "Manual"     // Optional
}
```

**Response** (200 OK):
```json
{
  "track_id": "abc123",
  "content": "[00:12.00]First line\n[00:16.50]Second line",
  "format": "lrc",
  "language": "en",
  "source": "Manual",
  "created_at": "2025-10-22T12:00:00Z",
  "updated_at": "2025-10-22T12:00:00Z"
}
```

**Error Responses**:
- `404 Not Found`: Track doesn't exist

### Delete Lyrics

```http
DELETE /lyrics/:id
```

**Response**: `204 No Content`

**Error Responses**:
- `404 Not Found`: Track doesn't exist or has no lyrics

## Lyric Formats

### Plain Text

Simple line-by-line lyrics without timestamps:

```
First line of the song
Second line of the song
Chorus begins here
```

### LRC Format

Synchronized lyrics with timestamps in `[mm:ss.xx]` format:

```
[00:12.00]First line of the song
[00:16.50]Second line of the song
[00:21.00]Chorus begins here
[00:25.30]Chorus continues
[00:30.00]
[00:35.50]Next verse
```

**LRC Format Details**:
- Timestamps: `[mm:ss.xx]` where mm=minutes, ss=seconds, xx=centiseconds
- Multiple timestamps per line supported: `[00:12.00][01:12.00]Same line repeated`
- Metadata tags supported (optional):
  - `[ti:Song Title]`
  - `[ar:Artist Name]`
  - `[al:Album Name]`
  - `[by:Creator]`
  - `[offset:+/-ms]` - Timing offset

## Usage Examples

### Using curl

**Upload plain text lyrics:**
```bash
curl -X PUT http://localhost:3000/lyrics/abc123 \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Line 1\nLine 2\nLine 3",
    "language": "en"
  }'
```

**Upload LRC lyrics:**
```bash
curl -X PUT http://localhost:3000/lyrics/abc123 \
  -H "Content-Type: application/json" \
  -d '{
    "content": "[00:12.00]First line\n[00:16.50]Second line",
    "format": "lrc",
    "language": "en",
    "source": "lyrics.com"
  }'
```

**Get lyrics:**
```bash
curl http://localhost:3000/lyrics/abc123
```

**Delete lyrics:**
```bash
curl -X DELETE http://localhost:3000/lyrics/abc123
```

### Using JavaScript (Web Client)

```javascript
// Upload lyrics
async function uploadLyrics(trackId, content) {
  const response = await fetch(`/lyrics/${trackId}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      content: content,
      language: 'en'
    })
  });
  return await response.json();
}

// Get lyrics
async function getLyrics(trackId) {
  const response = await fetch(`/lyrics/${trackId}`);
  if (response.ok) {
    return await response.json();
  }
  return null;
}

// Delete lyrics
async function deleteLyrics(trackId) {
  await fetch(`/lyrics/${trackId}`, {
    method: 'DELETE'
  });
}
```

## Track Integration

When you upload or delete lyrics, the track's `has_lyrics` field is automatically updated:

```json
{
  "id": "abc123",
  "title": "Song Name",
  "artist": "Artist Name",
  "has_lyrics": true,
  ...
}
```

This allows the web client to show a lyrics indicator next to tracks that have lyrics available.

## Database Location

The lyrics database is stored at:
```
<library-path>/.music-station/lyrics.db
```

For example, if your library is at `/music`, the database will be at:
```
/music/.music-station/lyrics.db
```

## Implementation Details

### Auto-format Detection

If you don't specify a format, the system automatically detects it:
- If content contains `[00:` or `[01:` patterns → LRC format
- Otherwise → Plain text format

### Timestamps

All timestamps are stored in RFC3339 format (ISO 8601):
```
2025-10-22T12:00:00Z
```

### Updates

When you upload lyrics for a track that already has lyrics, the existing lyrics are replaced (upsert operation). The `created_at` timestamp is preserved, but `updated_at` is updated.

## Future Enhancements

Potential features for future versions:

- **Batch operations**: Upload lyrics for multiple tracks
- **Search**: Find tracks by lyric content
- **Import/Export**: Import from .lrc files, export to various formats
- **Sync with online services**: Auto-fetch lyrics from online databases
- **Lyrics editor**: Visual editor in web client with LRC timestamp generator
- **Multi-language support**: Store multiple language versions per track
- **Karaoke mode**: Display synchronized lyrics during playback

## Troubleshooting

### Database initialization fails

If you see an error about database initialization:
1. Check that the library path exists and is writable
2. Verify `.music-station` directory can be created
3. Check disk space

### Lyrics not updating in track list

After uploading lyrics, refresh the track list:
```bash
curl http://localhost:3000/tracks
```

The `has_lyrics` flag should now be `true` for tracks with lyrics.

### Database file location

To manually inspect the database:
```bash
sqlite3 /path/to/library/.music-station/lyrics.db
sqlite> SELECT * FROM lyrics;
```
