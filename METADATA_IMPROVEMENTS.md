# Music Station - Metadata & Cover Art Improvements

## Overview
This document describes the comprehensive improvements made to the Music Station project to support extended metadata management and cover art functionality.

## Summary of Improvements

### 1. Extended Metadata Support

#### Backend (Rust)
- **Expanded Track Structure** (`src/library.rs`):
  - Added standard FLAC/Vorbis fields:
    - `album_artist` - Album artist (may differ from track artist)
    - `genre` - Music genre
    - `year` - Release year/date
    - `track_number` - Track number (e.g., "5" or "5/12")
    - `disc_number` - Disc number for multi-disc albums
    - `composer` - Composer name
    - `comment` - Additional comments/description
  - Added `custom_fields: HashMap<String, String>` for arbitrary metadata
  - Added `has_cover: bool` flag to indicate embedded cover art presence

- **Enhanced Metadata Parsing**:
  - Automatically extracts all standard FLAC/Vorbis comment tags
  - Collects unknown/custom tags into the `custom_fields` HashMap
  - Supports flexible metadata beyond predefined fields

- **Enhanced Metadata Writing**:
  - `write_flac_metadata()` now updates all standard fields
  - Supports writing custom fields to FLAC files
  - Preserves all existing metadata during updates

### 2. Cover Art Management

#### Backend Features
- **Cover Art Extraction** (`has_embedded_cover`, `get_cover_art`):
  - Detects embedded cover art in FLAC files
  - Extracts cover images as binary data
  
- **Cover Art API Endpoints** (`src/server.rs`):
  - `GET /cover/:id` - Retrieve cover art for a track
    - Auto-detects MIME type (JPEG/PNG)
    - Returns 404 if no cover exists
    - Includes cache headers for browser caching
  - `POST /cover/:id` - Upload new cover art
    - Accepts multipart form data with image file
    - Embeds image into FLAC file
    - Supports JPEG, PNG, and other formats
  - `DELETE /cover/:id` - Remove cover art from track

- **Cover Art Storage**:
  - Cover images are embedded directly in FLAC files
  - No separate image file storage needed
  - Standard FLAC picture block format

#### Frontend Features
- **Cover Art Display**:
  - Thumbnail column in track table showing album art
  - Fallback placeholder for tracks without covers
  - Error handling for missing/broken images

- **Cover Art Upload**:
  - File picker for selecting images
  - Live preview of selected image
  - Upload during metadata save operation
  - Remove cover button for deleting existing art

### 3. Enhanced Web UI

#### Improved Edit Modal
- **Organized Sections**:
  - Basic Information: Title, Artist, Album, Album Artist
  - Additional Details: Genre, Year, Track Number, Disc Number, Composer, Comment
  - Cover Art: Display current cover, upload new, or remove
  - Custom Fields: Add/edit/remove arbitrary metadata fields

- **Custom Fields Editor**:
  - Dynamic field addition with "Add Custom Field" button
  - Key-value pairs for any FLAC Vorbis comment
  - Remove individual fields
  - Persisted to FLAC file on save

#### Styling Improvements
- Modal expanded to `modal-wide` class (800px max-width)
- Form grid layout (2 columns) for better space usage
- Cover art preview with 150x150px display
- Custom field items with inline key/value inputs
- Responsive design maintained

### 4. Updated Dependencies
- Added `axum` multipart feature for file uploads
- Added `base64` crate for potential future image handling
- All dependencies remain compatible

## API Changes

### Track JSON Structure
```json
{
  "id": "abc123",
  "path": "/path/to/file.flac",
  "title": "Song Title",
  "artist": "Artist Name",
  "album": "Album Name",
  "album_artist": "Album Artist",
  "genre": "Rock",
  "year": "2024",
  "track_number": "5",
  "disc_number": "1",
  "composer": "Composer Name",
  "comment": "Some comment",
  "duration_secs": 245,
  "file_size": 25600000,
  "has_cover": true,
  "custom_fields": {
    "LABEL": "Record Label",
    "CATALOG": "CAT-12345"
  }
}
```

### Metadata Update Payload
```json
{
  "title": "New Title",
  "artist": "New Artist",
  "album": "New Album",
  "album_artist": "Album Artist",
  "genre": "Jazz",
  "year": "2024",
  "track_number": "1",
  "disc_number": "1",
  "composer": "John Doe",
  "comment": "Remastered",
  "custom_fields": {
    "LABEL": "Blue Note",
    "ISRC": "USXX12345678"
  }
}
```

## Usage Examples

### Viewing Extended Metadata
1. Navigate to Tracks tab
2. See cover art thumbnails in first column
3. Click "✏️" edit button on any track
4. All metadata fields are displayed in organized sections

### Editing Metadata
1. Open edit modal for a track
2. Fill in any standard fields (title, artist, genre, year, etc.)
3. Add custom fields with "➕ Add Custom Field" button
4. Enter custom field name (e.g., "LABEL") and value
5. Click "💾 Save Changes"
6. Changes are written to the FLAC file immediately

### Managing Cover Art
1. Open edit modal
2. Current cover (if any) displays in Cover Art section
3. Click "📷 Choose Image" to select an image file
4. Preview appears immediately
5. Save to embed in FLAC file
6. Click "🗑️ Remove Cover" to delete existing cover art

### Custom Fields Examples
Common custom FLAC tags you might add:
- `LABEL` - Record label name
- `CATALOG` - Catalog number
- `ISRC` - International Standard Recording Code
- `BARCODE` - UPC/EAN barcode
- `LYRICIST` - Lyricist name
- `ENCODER` - Software used for encoding
- `MOOD` - Mood/feel of the track
- `BPM` - Beats per minute

## Technical Implementation Details

### Cover Art Format
- Stored as FLAC picture metadata block
- Picture type: Front Cover
- Supports JPEG, PNG, and other image formats
- No size limits (though large images may slow operations)

### Custom Fields Storage
- Stored as FLAC Vorbis comments
- Key-value pairs with uppercase keys (convention)
- No reserved field name restrictions
- Any UTF-8 string values supported

### Performance Considerations
- Cover art loaded on-demand (not during initial scan)
- Browser caching reduces repeated image requests
- Custom fields parsed during initial scan (minimal overhead)
- Large images may increase file size and save time

## Future Enhancement Ideas
1. Batch metadata editing for multiple tracks
2. Cover art drag-and-drop upload
3. Automatic cover art download from online databases
4. Metadata templates for quick editing
5. Export/import metadata as JSON
6. Lyrics editor (as LYRICS custom field)
7. Image format conversion and optimization
8. Multiple cover art support (back cover, disc, etc.)

## Compatibility Notes
- All changes are backward compatible
- Existing FLAC files work without modification
- Cover art follows standard FLAC picture block format
- Custom fields use standard Vorbis comment format
- Works with any FLAC-compatible music player

## Testing Recommendations
1. Test with various FLAC files (with/without metadata)
2. Test cover art upload with different image formats
3. Test custom field creation and deletion
4. Verify metadata persists after server restart
5. Test with empty/missing fields
6. Verify FLAC files remain valid after editing
