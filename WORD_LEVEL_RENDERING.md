# Word-Level Lyrics Rendering Implementation

## Overview
Enhanced the web client to properly render word-level synchronized lyrics with karaoke-style highlighting. The implementation provides real-time word-by-word highlighting during playback, creating an immersive karaoke experience.

## Features

### 1. **Automatic Format Detection**
- Server now sends `format: "lrc_word"` for word-level lyrics
- Client automatically detects and uses appropriate rendering method
- Backward compatible with standard LRC format

### 2. **Word-Level Parsing**
New function `parseWordLevelLrcLyrics()` that:
- Preserves word timing data from format: `word(offset,duration)`
- Extracts individual word timestamps relative to line start time
- Creates data structure with both line and word timing information

### 3. **Karaoke-Style Rendering**
Two rendering locations:

#### **Lyrics Modal (View Tab)**
- Displays format badge: "🎤 Word-level synchronized lyrics (Karaoke mode)"
- Each word is wrapped in a `<span class="lyrics-word">` element
- Words contain data attributes for timing: `data-time`, `data-duration`, `data-line`, `data-word`

#### **Music Player**
- Real-time word highlighting during playback
- Smooth transitions between words
- Automatic line scrolling to keep current line centered

### 4. **Visual States**
Each word can have three states:

| State | Class | Appearance | Description |
|-------|-------|------------|-------------|
| **Future** | _(none)_ | Gray text (#555) | Words not yet sung |
| **Active** | `.active` | Orange gradient, larger, shadow | Currently singing |
| **Past** | `.sung` | Light gray (#999), faded | Already sung |

### 5. **Synchronized Highlighting**
Updated `updateSynchronizedLyrics()` function:
- Line-level highlighting (scrolls to center)
- Word-level highlighting within active line
- Smooth transitions using CSS transitions (0.15s)
- Clears highlighting from previous lines

## Technical Implementation

### Data Structure

```javascript
// Parsed line with word-level timing
{
    time: 12.34,              // Line start time (seconds)
    timestamp: "00:12.34",    // Formatted timestamp
    text: "original text",    // Original text with timing
    words: [                   // Array of words with timing
        {
            word: "word",      // Clean word text
            time: 12.50,       // Word start time (seconds)
            duration: 0.5      // Word duration (seconds)
        },
        // ... more words
    ],
    hasWordTiming: true       // Flag for word-level timing
}
```

### HTML Structure

```html
<!-- Word-level line -->
<div class="lyrics-line word-level-line" data-time="12.34" data-line="0">
    <span class="lyrics-timestamp">00:12.34</span>
    <span class="lyrics-text word-level-text">
        <span class="lyrics-word" data-time="12.50" data-duration="0.5" 
              data-line="0" data-word="0">word1</span>
        <span class="lyrics-word" data-time="13.00" data-duration="0.5" 
              data-line="0" data-word="1">word2</span>
        <!-- ... more words -->
    </span>
</div>
```

### CSS Styling

#### Modal Display
```css
.lyrics-word {
    display: inline-block;
    padding: 2px 4px;
    border-radius: 4px;
    transition: all 0.15s ease;
    color: #555;
    font-weight: 500;
}

.lyrics-word.active {
    background: linear-gradient(135deg, #702A03 0%, #D2691E 100%);
    color: white;
    font-weight: 700;
    transform: scale(1.1);
    box-shadow: 0 2px 8px rgba(210, 105, 30, 0.4);
}

.lyrics-word.sung {
    color: #999;
    opacity: 0.7;
}
```

#### Player Display
```css
.player-lyrics-content .lyrics-word.active {
    background: linear-gradient(135deg, #ff8c42 0%, #ffa366 100%);
    color: white;
    font-weight: 700;
    transform: scale(1.15);
    box-shadow: 0 2px 8px rgba(255, 140, 66, 0.5);
}
```

## Performance Optimizations

1. **Efficient DOM Queries**
   - Uses `data-` attributes for targeted selection
   - Minimal DOM manipulation per update

2. **CSS Transitions**
   - Hardware-accelerated transforms (`scale`)
   - Smooth 0.15s transitions without JavaScript

3. **Conditional Rendering**
   - Only processes word timing when format is `lrc_word`
   - Falls back to line-level rendering for standard LRC

## Usage Example

### Server Response
```json
{
    "content": "[0,11550]挪(0,721)威(721,721)的(1442,721)森(2163,721)林(2884,721)\n[11550,5000]Another(0,500) line(500,500)",
    "format": "lrc_word",
    "language": "zh",
    "source": "qqmusic"
}
```

### Rendered Output
When playing:
- Line 1 scrolls into view at 0s
- "挪" highlights at 0s, fades at 0.721s
- "威" highlights at 0.721s, fades at 1.442s
- ... continues word by word
- Line 2 scrolls into view at 11.55s
- Process repeats for line 2

## Browser Compatibility

- ✅ Modern browsers (Chrome 90+, Firefox 88+, Safari 14+)
- ✅ CSS Grid and Flexbox support required
- ✅ ES6+ JavaScript features (arrow functions, template literals)
- ⚠️ May degrade gracefully in older browsers (shows text without highlighting)

## Testing

### Manual Testing Steps
1. Start server: `cargo run -- --library /path/to/music`
2. Open web UI: `http://localhost:3000/web/`
3. Select a track and open lyrics modal
4. Search lyrics using QQ Music provider (has word-level timing)
5. Fetch and save word-level lyrics
6. Play the track and toggle lyrics display
7. Verify:
   - Format badge shows "Karaoke mode"
   - Words highlight in orange during playback
   - Past words fade to gray
   - Line scrolls to keep active line centered
   - Smooth transitions between words

### Expected Behavior
- **Line highlighting**: Active line has light orange background
- **Word highlighting**: Current word has bright orange background with scale effect
- **Past words**: Fade to gray after being sung
- **Scrolling**: Automatic smooth scroll keeps current line centered
- **Timing accuracy**: Words highlight within ±50ms of actual time

## Known Limitations

1. **Timing Accuracy**
   - Depends on `audio.currentTime` precision (typically ~10-50ms)
   - May drift slightly on long tracks

2. **Performance**
   - Very long songs (>1000 lines) may see slight lag
   - Consider pagination for extremely long lyrics

3. **Mobile Responsiveness**
   - Word highlighting may be less visible on small screens
   - Consider larger touch targets for mobile

## Future Enhancements

- [ ] Add toggle to switch between word-level and line-level display
- [ ] Support for right-to-left languages
- [ ] Customizable highlight colors in settings
- [ ] Export word-level timing to subtitle formats (ASS, SRT)
- [ ] Word-level editing interface
- [ ] Pitch/melody visualization alongside lyrics
- [ ] Offline lyrics caching for downloaded tracks

## Files Modified

### JavaScript (`static/app.js`)
- Added `parseWordLevelLrcLyrics()` - Parse and preserve word timing
- Added `parseWordsWithTiming()` - Extract individual word timing
- Updated `displayLyricsInModal()` - Render word-level lyrics in modal
- Updated `displayLyricsInPlayer()` - Render word-level lyrics in player
- Enhanced `updateSynchronizedLyrics()` - Add word-by-word highlighting logic

### CSS (`static/styles.css`)
- Added `.word-level-line` - Styling for word-level lines
- Added `.word-level-text` - Flexbox container for words
- Added `.lyrics-word` - Base word styling
- Added `.lyrics-word.active` - Active word highlighting
- Added `.lyrics-word.sung` - Past word fading
- Added `.lyrics-format-badge` - Format indicator badge
- Added player-specific word styles

## Integration Notes

### Backend Compatibility
- Requires server to send `format: "lrc_word"` for word-level lyrics
- Works with NetEase and QQ Music providers
- Automatic format detection via `LyricFormat::detect_from_content()`

### API Contract
```typescript
interface Lyrics {
    content: string;        // Lyrics content with timing
    format: 'plain' | 'lrc' | 'lrc_word';  // Format type
    language?: string;      // Language code
    source?: string;        // Provider name
}
```

## Debug Tips

### Check if Word Timing is Parsed
Open browser console:
```javascript
// After loading lyrics
console.log(lyricsLines);
// Look for 'words' array in each line
```

### Verify CSS Classes
```javascript
// During playback
document.querySelectorAll('.lyrics-word.active');
document.querySelectorAll('.lyrics-word.sung');
```

### Monitor Timing Updates
```javascript
// Hook into updateSynchronizedLyrics
const audio = document.getElementById('audioPlayer');
audio.addEventListener('timeupdate', () => {
    console.log('Current time:', audio.currentTime);
});
```
