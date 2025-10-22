# Lyrics Feature Migration Guide

## For Existing Music Station Users

The lyrics feature is fully backward compatible. Your existing music library and data will continue to work without any changes.

## What's New

A new SQLite database file will be created at:
```
<your-library-path>/.music-station/lyrics.db
```

This database is automatically created on first startup and stores all lyric data separately from your music files.

## Automatic Changes

### Track Structure
The `Track` JSON object now includes a new field:
```json
{
  "id": "...",
  "title": "...",
  ...
  "has_lyrics": false,  // NEW: indicates if track has lyrics
  ...
}
```

### New API Endpoints
Three new endpoints are available:
- `GET /lyrics/:id` - Get lyrics
- `PUT /lyrics/:id` - Upload/update lyrics
- `DELETE /lyrics/:id` - Delete lyrics

See [LYRICS_GUIDE.md](LYRICS_GUIDE.md) for details.

## No Breaking Changes

- All existing API endpoints work exactly as before
- The `has_lyrics` field is always set (defaults to `false`)
- No database migrations needed
- No configuration changes required

## Database Storage Location

The `.music-station` folder in your library directory will contain:
```
<library-path>/.music-station/
  └── lyrics.db         # SQLite database for lyrics
```

This folder is automatically created if it doesn't exist.

## Performance Impact

- Minimal: The lyrics database is queried only when needed
- Startup time: Slightly longer on first run (database initialization)
- Memory: No significant increase
- Disk: SQLite database size depends on number of lyrics (typically <1KB per track)

## Rollback

If you want to remove the lyrics feature:

1. Stop the server
2. Delete the lyrics database:
   ```bash
   rm <library-path>/.music-station/lyrics.db
   ```
3. The `has_lyrics` field will remain in API responses but will always be `false`

## Privacy & Data

- Lyrics are stored locally on your server
- No external services are contacted
- No telemetry or analytics
- Complete control over your data

## Upgrading

Simply pull the latest code and rebuild:
```bash
cd music-station
git pull
cargo build --release
```

The database will be created automatically on first run.
