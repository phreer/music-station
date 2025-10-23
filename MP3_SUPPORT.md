# MP3 Support

Music Station now supports both FLAC and MP3 audio files!

## What's New

### Audio Format Support
- ✅ **FLAC** - Full support (reading and writing metadata, cover art)
- ✅ **MP3** - Full support (reading and writing ID3v2 tags, cover art)

### Features Available for MP3

1. **Metadata Reading**
   - Title, Artist, Album, Album Artist
   - Genre, Year, Track Number, Disc Number
   - Comment field
   - Duration and file size

2. **Metadata Writing**
   - Update all standard ID3v2 tags
   - Edit metadata via REST API
   - Edit metadata via web interface

3. **Cover Art**
   - Extract embedded album art
   - Add/replace cover art
   - Remove cover art
   - Display in web interface

4. **Streaming**
   - HTTP streaming with range request support
   - Proper Content-Type header (`audio/mpeg`)
   - Seek support in web player

## Implementation Details

### Dependencies Added
```toml
symphonia = { version = "0.5", features = ["flac", "mp3"] }
id3 = "1.14"  # For MP3 metadata writing
```

### Key Changes

1. **Library Scanner** (`src/library.rs`)
   - Updated to scan for both `.flac` and `.mp3` files
   - Renamed `parse_flac_file()` to `parse_audio_file()` for generic audio parsing
   - Uses Symphonia for unified metadata extraction

2. **Metadata Writing**
   - `write_flac_metadata()` - Uses `metaflac` crate for FLAC files
   - `write_mp3_metadata()` - Uses `id3` crate for MP3 files (ID3v2.4)
   - `write_audio_metadata()` - Dispatcher that routes to the correct writer

3. **Cover Art Handling**
   - `has_embedded_cover()` - Checks both FLAC and MP3 formats
   - `get_cover_art()` - Extracts cover from either format
   - `set_cover_art()` - Adds cover to either format
   - `remove_cover_art()` - Removes cover from either format

4. **HTTP Streaming** (`src/server.rs`)
   - Content-Type detection: `audio/flac` or `audio/mpeg`
   - Range request support for both formats

## Usage Examples

### Server
```bash
# Scan a folder with both FLAC and MP3 files
cargo run -- --library /path/to/music

# The server will automatically detect and process both formats
```

### API
```bash
# Get track info (works for both FLAC and MP3)
curl http://localhost:3000/tracks

# Stream MP3 file
curl http://localhost:3000/stream/<track-id> -o song.mp3

# Update MP3 metadata
curl -X PUT http://localhost:3000/tracks/<track-id> \
  -H "Content-Type: application/json" \
  -d '{"title":"New Title","artist":"New Artist"}'
```

### Web Client
- Open `http://localhost:3000/web/index.html`
- MP3 files appear alongside FLAC files
- All features work identically for both formats
- Cover art thumbnails display for both formats

## Technical Notes

### ID3v2 vs Vorbis Comments
- **FLAC** uses Vorbis comments (flexible key-value pairs)
- **MP3** uses ID3v2 frames (standardized frame types)
- Custom fields are more limited in MP3 compared to FLAC

### ID3v2 Frame Mappings
| Field | ID3v2 Frame |
|-------|-------------|
| Title | TIT2 |
| Artist | TPE1 |
| Album | TALB |
| Album Artist | TPE2 |
| Genre | TCON |
| Year | TDRC |
| Track Number | TRCK |
| Disc Number | TPOS |
| Composer | TCOM |
| Comment | COMM |

### Symphonia Format Detection
Symphonia automatically detects the audio format based on:
1. File extension hint (`.flac` or `.mp3`)
2. File magic bytes/signature
3. Codec probing

This allows seamless handling of mixed format libraries.

## Limitations

1. **MP3 Custom Fields**
   - ID3v2 doesn't have as flexible custom field support as FLAC
   - Custom fields would need to use TXXX frames (not currently implemented)

2. **MP3 Composer Field**
   - Basic composer support via TCOM frame
   - Not all players support this tag

3. **File Format Mixing**
   - Metadata schemas differ between formats
   - Some fields may not map perfectly between FLAC and MP3

## Testing

The implementation has been tested with:
- ✅ Library scanning with mixed FLAC/MP3 files
- ✅ Metadata reading from both formats
- ✅ Metadata writing to both formats
- ✅ Cover art extraction and embedding
- ✅ HTTP streaming with correct Content-Type
- ✅ Web client playback

## Future Enhancements

Potential improvements:
- [ ] Support for more formats (AAC, OGG, OPUS)
- [ ] Advanced ID3v2 tag editing (lyrics, multiple pictures)
- [ ] TXXX frame support for MP3 custom fields
- [ ] Batch format conversion
- [ ] ReplayGain tag support
