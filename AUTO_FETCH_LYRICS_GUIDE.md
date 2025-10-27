# Auto-Fetch Lyrics Guide

## Overview

The Auto-Fetch Lyrics feature automatically searches for and downloads lyrics for all tracks in your library that don't currently have lyrics. It uses the QQ Music API to search for matching lyrics and applies them to your tracks.

## Features

- **Bulk Processing**: Automatically processes all tracks without lyrics
- **Smart Matching**: Matches tracks based on duration (within 10 seconds)
- **Progress Tracking**: Real-time progress display with detailed statistics
- **Rate Limiting**: 1-second delay between requests to avoid overloading the API
- **Cancellable**: Can be cancelled at any time
- **Detailed Logging**: Shows success/failure/skip status for each track

## How to Use

### Starting Auto-Fetch

1. Open the Music Station web client at `http://localhost:3000/web/index.html`
2. Navigate to the **Tracks** tab (should be active by default)
3. Click the **🎵 Auto-Fetch Lyrics** button in the toolbar
4. A confirmation dialog will show the number of tracks without lyrics
5. Click **OK** to start the process

### During Auto-Fetch

The auto-fetch modal will display:

- **Progress Bar**: Visual progress indicator with percentage
- **Statistics**:
  - Total tracks without lyrics
  - Processed count
  - Succeeded count (lyrics found and applied)
  - Failed count (errors during processing)
  - Skipped count (no match found or duration mismatch)
- **Current Track**: Shows which track is currently being processed
- **Log**: Detailed log of all operations with timestamps

### Cancelling Auto-Fetch

- Click the **⏸️ Cancel** button to stop the process
- Tracks that have already been processed will retain their lyrics
- You can close the modal after cancellation

### After Completion

- Click **✓ Close** to close the modal
- The track list will automatically refresh to show updated lyrics status
- Tracks with newly added lyrics will show the lyrics icon (📄)

## How It Works

1. **Identify Tracks**: Filters all tracks to find those without lyrics (`has_lyrics: false`)
2. **Search for Each Track**:
   - Uses track title and artist to search QQ Music
   - Searches using the endpoint: `GET /lyrics/search?q={title}&provider=qqmusic&artist={artist}`
3. **Match by Duration**:
   - Compares search results with the track's duration
   - Accepts matches where the duration difference is **less than 10 seconds**
   - Chooses the result with the smallest duration difference
4. **Fetch Lyrics**:
   - Retrieves lyrics from QQ Music: `GET /lyrics/fetch/qqmusic/{song_id}`
   - Returns lyrics in plain, LRC (line-level), or LRC_WORD (word-level) format
5. **Apply Lyrics**:
   - Uploads lyrics to the track: `PUT /lyrics/{track_id}`
   - Updates the track's `has_lyrics` flag to `true`
6. **Rate Limiting**:
   - Waits 1 second between processing each track
   - Prevents overloading the QQ Music API

## Result Categories

### ✓ Success
Lyrics were found, matched by duration, and successfully applied to the track.

### ✗ Error
An error occurred during search, fetch, or upload. Common causes:
- Network issues
- API unavailable
- Invalid track data

### ⊘ Skipped (No Results)
No search results were found for the track title/artist combination.

### ⊘ Skipped (Duration Mismatch)
Search results were found but none matched the track's duration (within 10 seconds).

## Tips for Best Results

1. **Ensure Accurate Metadata**: 
   - Make sure track titles and artists are correct
   - Edit tracks with incorrect metadata before running auto-fetch

2. **Internet Connection**:
   - Stable internet connection required
   - Process may take time for large libraries

3. **Duration Tolerance**:
   - 10-second duration tolerance works well for most tracks
   - Live recordings or different versions may be skipped

4. **Manual Review**:
   - After auto-fetch, manually review skipped tracks
   - Use the manual lyrics search for tracks that were skipped

5. **Multiple Runs**:
   - Safe to run multiple times
   - Only processes tracks without lyrics
   - Won't overwrite existing lyrics

## Limitations

- **QQ Music Only**: Currently only uses QQ Music API (can be extended to support NetEase)
- **Duration-Based Matching**: May skip alternate versions (live, acoustic, remixes)
- **Rate Limiting**: 1-second delay means large libraries take time (1000 tracks ≈ 17 minutes)
- **No Manual Selection**: Automatically chooses the best match (no manual verification)

## Troubleshooting

### No Tracks Found
- Ensure some tracks actually lack lyrics (check tracks tab for 📄 icon)
- Refresh the library first

### High Skip Rate
- Check that track metadata (title, artist) is accurate
- Duration mismatches may indicate incorrect metadata or different versions

### Errors During Processing
- Check server logs: `RUST_LOG=debug cargo run -- --library /path/to/music`
- Verify QQ Music API is accessible
- Check network connection

### Process Hangs
- Click Cancel to stop
- Check browser console for JavaScript errors
- Refresh page and try again

## Technical Details

### API Endpoints Used

1. **Search Lyrics**:
   ```
   GET /lyrics/search?q={query}&provider=qqmusic&artist={artist}
   ```

2. **Fetch Lyrics**:
   ```
   GET /lyrics/fetch/qqmusic/{song_id}
   ```

3. **Upload Lyrics**:
   ```
   PUT /lyrics/{track_id}
   Body: { content, format, language, source }
   ```

### Duration Matching Logic

```javascript
const durationDiff = Math.abs(result.duration.secs - track.duration_secs);
if (durationDiff < 10 && durationDiff < minDurationDiff) {
    bestMatch = result;
    minDurationDiff = durationDiff;
}
```

### Rate Limiting

```javascript
await sleep(1000); // 1 second between requests
```

## Future Enhancements

Potential improvements for the auto-fetch feature:

- [ ] Support for NetEase Cloud Music provider
- [ ] Configurable duration tolerance
- [ ] Manual confirmation before applying lyrics
- [ ] Parallel processing with rate limiting
- [ ] Retry logic for failed requests
- [ ] Preview lyrics before applying
- [ ] Match by additional metadata (album, year)
- [ ] Save unmatched tracks for manual review
- [ ] Export/import auto-fetch results

## Related Documentation

- [Lyrics Guide](LYRICS_GUIDE.md) - Complete lyrics system documentation
- [Lyrics API](LYRICS_API.md) - REST API documentation for lyrics
- [Web Client Guide](WEB_CLIENT_GUIDE.md) - Web interface documentation
- [API Documentation](API_DOCUMENTATION.md) - Full REST API reference

---

**Last Updated**: October 27, 2025
