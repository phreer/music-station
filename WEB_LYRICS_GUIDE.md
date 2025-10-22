# Web Client Lyrics Feature Guide

## Overview

The Music Station web client now includes comprehensive lyrics support, allowing you to view, edit, and synchronize lyrics with your music playback. The feature supports both plain text and LRC (synchronized) lyrics formats.

## Features

### 1. **Lyrics Management**
- ✅ View lyrics for any track
- ✅ Add/edit lyrics through an intuitive modal interface
- ✅ Delete lyrics when no longer needed
- ✅ Support for plain text and LRC synchronized formats
- ✅ Auto-detection of LRC format
- ✅ Optional metadata (language, source)

### 2. **Synchronized Playback**
- ✅ Real-time lyrics synchronization during playback
- ✅ Auto-scroll to current line
- ✅ Highlight active lyrics line
- ✅ Smooth transitions between lines

### 3. **Visual Indicators**
- ✅ Lyrics button shows 📝 icon when lyrics exist
- ✅ Lyrics button shows 📄 icon when no lyrics
- ✅ Color-coded buttons (primary for tracks with lyrics)
- ✅ Lyrics toggle button appears during playback when lyrics available

## Using the Lyrics Feature

### Viewing Lyrics

1. **From Track List:**
   - Click the lyrics button (📝 or 📄) next to any track
   - The lyrics modal will open showing the current lyrics (if any)
   - Use the "View" tab to see formatted lyrics

2. **During Playback:**
   - When playing a track with lyrics, a "📝 Lyrics" button appears in the bottom-right
   - Click it to open the floating lyrics panel
   - Lyrics will auto-scroll and highlight the current line

### Adding Lyrics

1. Click the lyrics button (📄) for a track without lyrics
2. Switch to the "Edit" tab in the modal
3. Enter or paste your lyrics in the text area
4. **For plain text lyrics:**
   ```
   First verse line one
   First verse line two
   
   Chorus line one
   Chorus line two
   ```

5. **For synchronized LRC lyrics:**
   ```
   [00:12.00]First verse line one
   [00:16.50]First verse line two
   [00:21.00]Chorus line one
   [00:25.30]Chorus line two
   ```

6. (Optional) Select format, add language (e.g., "en"), and source
7. Click "💾 Save Lyrics"

### Editing Lyrics

1. Click the lyrics button (📝) for a track with lyrics
2. View the current lyrics in the "View" tab
3. Switch to the "Edit" tab to make changes
4. Modify the lyrics content
5. Click "💾 Save Lyrics" to update

### Deleting Lyrics

1. Open the lyrics modal for a track with lyrics
2. Switch to the "Edit" tab
3. Click the "🗑️ Delete Lyrics" button
4. Confirm the deletion

## LRC Format Guide

### Basic Format

LRC (Lyrics) format uses timestamps to synchronize lyrics with audio:

```
[mm:ss.xx]Lyric text
```

- `mm` = minutes (00-99)
- `ss` = seconds (00-59)
- `xx` = centiseconds (00-99)

### Example

```
[00:12.00]When the night has come
[00:16.50]And the land is dark
[00:20.00]And the moon is the only light we'll see
[00:28.00]No I won't be afraid
[00:32.50]Oh, I won't be afraid
[00:36.00]Just as long as you stand, stand by me
```

### Metadata Tags (Optional)

You can include metadata at the top of LRC files:

```
[ti:Song Title]
[ar:Artist Name]
[al:Album Name]
[by:Creator]
[offset:+/-ms]
[00:12.00]First line of lyrics...
```

### Creating LRC Timestamps

1. **Manual Method:**
   - Play the track
   - Note the timestamp when each line appears
   - Format: `[MM:SS.CC]Lyric line`

2. **Using Tools:**
   - Use online LRC editors (many free options available)
   - Desktop LRC editors with audio playback
   - Some media players have built-in LRC editors

### Tips for Good LRC Lyrics

- Place timestamps at the beginning of each phrase/line
- Include empty lines for instrumental breaks: `[01:30.00]`
- Be precise with timing (within 0.5 seconds is good)
- Test with playback to verify synchronization
- Consider adding a small offset if consistently early/late

## Lyrics Panel Features

### During Playback

When playing a track with lyrics:

1. **Toggle Button:** Click "📝 Lyrics" to show/hide the lyrics panel
2. **Auto-Scroll:** Lyrics automatically scroll to keep current line centered
3. **Highlighting:** Current line is highlighted with a colored background
4. **Timestamps:** LRC lyrics show timestamps next to each line
5. **Smooth Animation:** Transitions are smooth and non-distracting

### Panel Position

- **Desktop:** Floating panel in bottom-right corner
- **Mobile:** Full-width panel at bottom of screen
- **Always On Top:** Panel stays visible while browsing other tabs

## Keyboard Shortcuts

- **Esc**: Close lyrics modal (when open)
- Click outside modal to close

## Format Auto-Detection

The system automatically detects LRC format when:
- Content contains `[00:` or `[01:` patterns
- You can override by manually selecting format

Auto-detected formats:
- Contains `[MM:SS.CC]` → LRC format
- Plain text → Plain format

## Tips & Best Practices

### For Best Experience

1. **Use LRC for Synchronized Experience:**
   - Provides karaoke-like scrolling
   - More engaging during playback
   - Professional appearance

2. **Plain Text for Quick Entry:**
   - Faster to add
   - Good for reference
   - Works when exact timing isn't available

3. **Add Metadata:**
   - Language helps with multi-language collections
   - Source helps track where lyrics came from
   - Useful for organizing large libraries

4. **Quality Lyrics:**
   - Check spelling and grammar
   - Match original artist's lyrics
   - Include proper punctuation
   - Use line breaks to match song phrasing

### Organization Tips

1. **Consistent Language Tags:**
   - Use ISO 639-1 codes (en, es, fr, ja, etc.)
   - Makes filtering easier in the future

2. **Source Tracking:**
   - Note where lyrics came from
   - Examples: "Manual", "lyrics.com", "Album booklet"
   - Helps verify accuracy later

3. **Regular Updates:**
   - Review lyrics periodically
   - Fix any errors found during playback
   - Update timestamps if they drift

## Troubleshooting

### Lyrics Not Showing

1. **Check if lyrics exist:**
   - Look for 📝 icon (has lyrics) vs 📄 icon (no lyrics)
   - Click the button to open modal and verify

2. **Refresh track list:**
   - Click "🔄 Refresh Library" button
   - Reload the page

3. **Check console:**
   - Press F12 to open developer tools
   - Look for errors in Console tab

### Synchronization Issues

1. **Lyrics appear too early/late:**
   - Add offset in LRC: `[offset:+500]` (adds 500ms delay)
   - Or manually adjust all timestamps

2. **Lyrics don't scroll:**
   - Ensure format is set to "LRC"
   - Verify timestamps are valid: `[MM:SS.CC]`
   - Check browser console for errors

3. **Wrong line highlighted:**
   - Review timestamp accuracy
   - Ensure format is `[00:12.34]` not `[0:12.34]` or `[00:12:34]`

### Display Issues

1. **Text too small/large:**
   - Use browser zoom (Ctrl/Cmd +/-)
   - CSS is responsive and should adapt

2. **Panel won't close:**
   - Click the ✖️ button
   - Click "📝 Lyrics" toggle button again

3. **Modal won't open:**
   - Check if another modal is already open
   - Press Esc to close any open modals
   - Try refreshing the page

## Advanced Usage

### Batch Adding Lyrics

1. Use the API directly with curl/scripts:
   ```bash
   for track_id in $(cat track_ids.txt); do
     curl -X PUT "http://localhost:3000/lyrics/$track_id" \
       -H "Content-Type: application/json" \
       -d @"lyrics/${track_id}.json"
   done
   ```

2. Create JSON files with lyrics:
   ```json
   {
     "content": "[00:12.00]Line 1\n[00:16.50]Line 2",
     "format": "lrc",
     "language": "en",
     "source": "Batch import"
   }
   ```

### Exporting Lyrics

Use the API to export lyrics:
```bash
# Export all lyrics
curl http://localhost:3000/tracks | \
  jq -r '.[] | select(.has_lyrics) | .id' | \
  while read id; do
    curl "http://localhost:3000/lyrics/$id" > "lyrics_${id}.json"
  done
```

### Custom LRC Editors

Popular free LRC editors:
- **Online:** LRC Editor, Lyrics Editor
- **Desktop:** MiniLyrics, LRC Maker
- **Mobile:** LRC Editor apps on iOS/Android

## API Reference (Quick)

For developers integrating with the lyrics feature:

```javascript
// Get lyrics
const lyrics = await fetch(`/lyrics/${trackId}`).then(r => r.json());

// Upload lyrics
await fetch(`/lyrics/${trackId}`, {
  method: 'PUT',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    content: lyricsText,
    format: 'lrc',  // or 'plain'
    language: 'en',
    source: 'Manual'
  })
});

// Delete lyrics
await fetch(`/lyrics/${trackId}`, { method: 'DELETE' });
```

## Future Enhancements

Planned features for future updates:

- [ ] Visual LRC timestamp editor
- [ ] Import .lrc files directly
- [ ] Search lyrics content
- [ ] Lyrics translation support
- [ ] Karaoke mode with fullscreen
- [ ] Export lyrics to .lrc files
- [ ] Lyrics fetching from online databases
- [ ] Multiple language versions per track

## Support

For issues or questions:
1. Check this guide first
2. Review the main LYRICS_GUIDE.md for API details
3. Check browser console for error messages
4. Open an issue on GitHub with details

## Credits

Lyrics feature supports:
- Plain text lyrics (universal)
- LRC format (standard synchronized lyrics)
- Compatible with most LRC editors and players
