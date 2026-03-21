# Lyrics Feature Guide

## Overview

Music Station now supports lyrics storage and management for your music library. Lyrics are stored in a SQLite database and can be uploaded, retrieved, and deleted through REST API endpoints.

## Features

- **Persistent Storage**: Lyrics are stored in a SQLite database at `.music-station/lyrics.db` in your library folder
- **Multiple Formats**: Supports plain text, LRC (line-synchronized), and LRC-Word (word-level synchronized) formats
- **Metadata Support**: Store language, source, and timestamps with lyrics
- **Track Integration**: The `has_lyrics` flag is automatically updated on tracks
- **Auto-format Detection**: Automatically detects LRC format based on content

## Database Schema

The lyrics database contains a single table:

```sql
CREATE TABLE lyrics (
    track_id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    format TEXT NOT NULL,       -- 'plain', 'lrc', or 'lrc_word'
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
  "format": "lrc",       // Optional: "plain", "lrc", or "lrc_word" (auto-detected if omitted)
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

The system supports three formats, stored in the `format` field as a string enum.

### 1. Plain Text (`plain`)

Simple line-by-line lyrics without timestamps:

```
First line of the song
Second line of the song
Chorus begins here
```

### 2. LRC Format (`lrc`)

Synchronized lyrics with line-level timestamps. Two timestamp syntaxes are supported:

**Standard LRC** — `[mm:ss.xx]` where mm=minutes, ss=seconds, xx=centiseconds or milliseconds:

```
[00:12.00]First line of the song
[00:16.50]Second line of the song
[00:21.00]Chorus begins here
```

**Offset LRC** — `[offset,duration]` where both values are in milliseconds. Common in lyrics fetched from NetEase/QQ Music providers:

```
[0,12210]First line of the song
[12210,5000]Second line of the song
[48853,2103]Third line of the song
```

In offset format, the first number is the absolute start time in ms, and the second is the line duration in ms.

**LRC metadata tags** (optional, skipped during parsing):
- `[ti:Song Title]`
- `[ar:Artist Name]`
- `[al:Album Name]`
- `[by:Creator]`
- `[offset:+/-ms]` — Timing offset

### 3. LRC-Word Format (`lrc_word`)

Word-level synchronized lyrics. Each line has a line-level timestamp (either standard or offset format), and within each line, individual words have their own timing annotation:

```
word(absolute_offset_ms,duration_ms)
```

**Example with offset line timestamps** (most common from providers):

```
[ti:双刀]
[ar:周杰伦]
[al:叶惠美]
[by:]
[offset:0]
[0,12210]双(0,872)刀(872,872) (1744,872)-(2616,872) (3488,872)周(4360,872)杰(5232,872)伦(6104,872)
[48853,2103]透(48853,155)过(49008,338)镜(49346,152)头(49498,230)
```

**Example with standard line timestamps**:

```
[00:48.85]透(48853,155)过(49008,338)镜(49346,152)头(49498,230)
```

**Word timing semantics**:
- `word(offset,duration)` — `offset` is the absolute start time in milliseconds from the beginning of the track, `duration` is how long the word is sung in milliseconds
- Clients convert to seconds by dividing by 1000
- Words within a line are concatenated (no implicit spaces) — whitespace characters appear as literal text between word groups

### Auto-Detection Logic

The server auto-detects format when none is specified (see `src/lyrics.rs`):
1. If content matches word-level timing pattern `word(offset,duration)` → `lrc_word`
2. If content contains standard LRC timestamps `[mm:ss.xx]` or offset timestamps `[offset,duration]` → `lrc`
3. Otherwise → `plain`

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

See the "Auto-Detection Logic" section under "Lyric Formats" above for details on how format is determined when not explicitly specified.

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
