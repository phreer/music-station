# Metadata & Cover Art Quick Start Guide

## New Features at a Glance

### 📋 Extended Metadata Fields
You can now edit these fields for each track:
- **Title, Artist, Album** - Basic track information
- **Album Artist** - For compilation albums
- **Genre** - Music genre (Rock, Jazz, Classical, etc.)
- **Year** - Release year
- **Track Number** - Track position (e.g., "5" or "5/12")
- **Disc Number** - For multi-disc albums
- **Composer** - Composer name
- **Comment** - Additional notes

### 🎨 Cover Art Support
- **View** cover art thumbnails in the track list
- **Upload** custom cover images (JPEG, PNG)
- **Remove** existing cover art
- Cover art is embedded in the FLAC file

### 🔧 Custom Fields
Add any custom metadata you want:
- Label name, catalog numbers
- ISRC codes, barcodes
- Mood tags, BPM values
- Any other information you need

## Quick How-To

### Edit Track Metadata
1. Click the **✏️** button next to any track
2. Fill in the fields you want to change
3. Click **💾 Save Changes**

### Add Cover Art
1. Open the track editor
2. In the "Cover Art" section, click **📷 Choose Image**
3. Select an image file from your computer
4. Click **💾 Save Changes** to embed it

### Add Custom Fields
1. Open the track editor
2. Scroll to "Custom Fields" section
3. Click **➕ Add Custom Field**
4. Enter field name (e.g., "LABEL") and value
5. Click **💾 Save Changes**

### View Cover Art
- Cover thumbnails appear in the leftmost column of the track table
- Click on album cards to see tracks grouped by album
- Missing covers show a placeholder icon

## API Endpoints

For developers and advanced users:

```bash
# Get cover art
GET /cover/:track_id

# Upload cover art (multipart form data)
POST /cover/:track_id

# Remove cover art
DELETE /cover/:track_id

# Update metadata (JSON)
PUT /tracks/:track_id
```

## Tips & Tricks

### Metadata Best Practices
- Use consistent naming conventions (e.g., always "Rock" not "rock")
- Track numbers: Use "5" or "5/12" format
- Years: Use 4-digit format (2024) or full date (2024-03-15)
- Album Artist: Use for compilations (e.g., "Various Artists")

### Cover Art Guidelines
- Recommended size: 500x500 to 1400x1400 pixels
- Format: JPEG for smaller file size, PNG for quality
- Square aspect ratio works best
- Avoid very large files (>5MB) to keep FLAC files manageable

### Custom Field Ideas
- `LABEL` - Record label
- `CATALOG` - Catalog number  
- `ISRC` - International Standard Recording Code
- `BARCODE` - UPC/EAN barcode
- `LYRICIST` - Lyricist name
- `MOOD` - Track mood/feel
- `BPM` - Beats per minute
- `LANGUAGE` - Lyrics language
- `ENCODER` - Encoding software
- `REPLAYGAIN_*` - ReplayGain values

## Troubleshooting

**Q: Cover art not showing?**
- Check that the image file is valid JPEG or PNG
- Try refreshing the browser (Ctrl+R or Cmd+R)
- Check browser console for errors (F12)

**Q: Metadata changes not saved?**
- Ensure fields have valid values
- Check file permissions (server needs write access)
- Look for error messages in the browser

**Q: Custom fields not appearing?**
- Field names should be uppercase (convention)
- Both name and value must be filled in
- Click Save Changes after adding fields

**Q: How do I remove a field?**
- For custom fields: Click the ❌ button next to the field
- For standard fields: Clear the text and save (saves as empty)

## Keyboard Shortcuts

In edit modal:
- **Esc** - Close modal without saving
- **Tab** - Navigate between fields
- **Enter** - Submit form (save changes)

## Browser Compatibility

Tested and working on:
- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

## Need Help?

- Check the main README.md for general usage
- See METADATA_IMPROVEMENTS.md for technical details
- File issues on GitHub for bugs or feature requests
