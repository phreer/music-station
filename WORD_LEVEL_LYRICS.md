# Word-Level Lyrics Format Support

## Overview
Added support for word-level synchronized lyrics format (extended LRC format) that includes both line-level and word-level timestamps. This format is commonly returned by QQ Music API.

## Format Examples

### Standard LRC (Line-Level)
```
[00:12.34]This is a line of lyrics
[00:16.78]Another line follows
```

### Extended LRC (Word-Level)
```
[0,11550]挪(0,721)威(721,721)的(1442,721)森(2163,721)林(2884,721)
[11550,5000]Another(0,500) line(500,500)
```

The word-level format includes:
- Line timestamp: `[offset_ms,duration_ms]`
- Word timing: `word(offset_ms_from_line_start,duration_ms)`

## Server-Side Changes

### 1. LyricFormat Enum (`src/lyrics.rs`)

Added a new variant to the `LyricFormat` enum:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LyricFormat {
    Plain,
    Lrc,      // Standard LRC (line-level timestamps)
    #[serde(rename = "lrc_word")]
    LrcWord,  // Extended LRC (word-level timestamps)
}
```

**Key Methods:**

- `as_str()`: Returns format string ("plain", "lrc", or "lrc_word")
- `from_str()`: Parses format from string, supports aliases ("lrc_word", "lrcword", "word", "extended")
- `detect_from_content()`: **NEW** - Automatically detects format from lyrics content

### 2. Format Detection Logic

The `detect_from_content()` method analyzes lyrics content and determines the format:

```rust
pub fn detect_from_content(content: &str) -> Self {
    // Check for word-level timing: word(offset,duration)
    if content matches word_timing_pattern {
        return LyricFormat::LrcWord;
    }
    
    // Check for line-level LRC: [mm:ss.xx] or [offset,duration]
    if content matches lrc_pattern {
        return LyricFormat::Lrc;
    }
    
    // Default to plain text
    LyricFormat::Plain
}
```

**Detection Patterns:**
- Word-level: `\S+\(\d+,\d+\)` (e.g., `word(123,456)`)
- LRC: `\[\d+:\d{2}\.\d{2,3}\]` or `\[\d+,\d+\]`

### 3. Provider Updates (`src/lyrics/music_search_provider.rs`)

Both NetEase and QQ Music providers now use automatic format detection:

```rust
// Old code (manual detection):
let format = if content.contains("[00:") || content.contains("[01:") {
    LyricFormat::Lrc
} else {
    LyricFormat::Plain
};

// New code (automatic detection):
let format = LyricFormat::detect_from_content(&content);
```

This allows QQ Music's word-level lyrics to be properly recognized as `LrcWord` format.

## Client-Side Changes

### 1. JavaScript Parsing (`static/app.js`)

Added functions to handle word-level lyrics:

```javascript
// Parse LRC format (handles both standard and extended formats)
function parseLrcLyrics(content) {
    const lineRegex = /\[(\d+):(\d{2})\.(\d{2,3})\](.+)|(\[\d+,\d+\])(.+)/g;
    // ... parsing logic
}

// Extract plain text from word-level format
function parseWordLevelLyrics(content) {
    // Strips word timing: word(123,456) -> word
    return content.replace(/\((\d+),(\d+)\)/g, '');
}
```

### 2. Visual Indicators

The UI now displays format badges:

```javascript
if (/\S+\(\d+,\d+\)/.test(content)) {
    formatBadge = '<span class="format-badge">Word-Level</span>';
}
```

Badges show:
- "Plain" - Plain text lyrics
- "LRC" - Line-level synchronized lyrics
- "Word-Level" - Word-level synchronized lyrics

## Database Compatibility

The `format` column in the `lyrics` table is stored as TEXT, so it can store:
- "plain"
- "lrc"
- "lrc_word"

No schema migration is required. Existing lyrics with "lrc" format remain valid.

## API Response Format

The API endpoints now return the detected format:

```json
{
    "content": "[0,11550]挪(0,721)威(721,721)的(1442,721)森(2163,721)林(2884,721)",
    "format": "lrc_word",
    "language": "zh",
    "source": "qqmusic",
    "url": "https://y.qq.com/n/ryqq/songDetail/123456"
}
```

## Testing

Test the word-level format detection:

1. Search for a song using QQ Music provider
2. Fetch lyrics for a song with word-level timing
3. Verify the format is detected as "lrc_word" in the API response
4. Check that the web client displays the "Word-Level" badge
5. Verify that clicking "Display Plain Text" strips word timing correctly

## Dependencies Added

- `regex = "1.12.2"` - Required for format detection pattern matching

## Future Enhancements

Potential improvements:
- Render word-level timing in the player (karaoke-style highlighting)
- Support word-level editing in the lyrics editor
- Convert between formats (word-level ↔ line-level ↔ plain)
- Store word timing separately for more efficient parsing
