# Music Search Binary - Usage Guide

## Overview

The `music_search` binary is an interactive command-line tool for searching songs and downloading lyrics from NetEase Cloud Music and QQ Music. It includes comprehensive tracing/logging capabilities for debugging and monitoring.

## Building

### Release Build (Recommended)
```bash
cargo build --bin music_search --release
```

The binary will be created at `target/release/music_search` (approximately 5.1MB).

### Debug Build
```bash
cargo build --bin music_search
```

The binary will be created at `target/debug/music_search`.

## Installation

You can install the binary to your system:

```bash
# Install to ~/.cargo/bin (must be in your PATH)
cargo install --path . --bin music_search

# Or copy the binary manually
sudo cp target/release/music_search /usr/local/bin/
```

## Usage

### Running the Binary

```bash
# If installed
music_search

# Or run directly
./target/release/music_search

# With command-line arguments
./target/release/music_search --api qq --query "告白气球"

# With cookie authentication
./target/release/music_search --api netease --cookie "YOUR_COOKIE" --query "周杰伦"

# Using environment variable for cookie
export MUSIC_COOKIE="YOUR_COOKIE"
./target/release/music_search --api netease --query "稻香"
```

### Command-Line Options

```
Usage: music_search [OPTIONS]

Options:
  -a, --api <SERVICE>    Music service to use: 'netease' or 'qq'
  -c, --cookie <COOKIE>  Cookie for authentication (can also be set via MUSIC_COOKIE env var)
  -q, --query <QUERY>    Search query (song name or artist)
  -h, --help             Print help
  -V, --version          Print version
```

#### API Service Options

You can specify the music service using any of these values:

**NetEase Cloud Music:**
- `netease`
- `ne`
- `163`
- `1`

**QQ Music:**
- `qq`
- `qqmusic`
- `tencent`
- `2`

#### Cookie Authentication

For services that require authentication (like NetEase Cloud Music search), you can provide a cookie:

**Method 1: Command-line argument**
```bash
./target/release/music_search --api netease --cookie "MUSIC_U=your_cookie_value"
```

**Method 2: Environment variable**
```bash
export MUSIC_COOKIE="MUSIC_U=your_cookie_value"
./target/release/music_search --api netease
```

**Method 3: .env file (if supported)**
```bash
echo 'MUSIC_COOKIE="MUSIC_U=your_cookie_value"' > .env
./target/release/music_search --api netease
```

## Logging and Debugging

The binary uses the `tracing` framework for structured logging. Control log output with the `RUST_LOG` environment variable.

### Log Levels

- **error**: Only critical errors
- **warn**: Warnings and errors (default)
- **info**: Informational messages (API calls, success/failure)
- **debug**: Detailed information (request/response sizes, parameters)
- **trace**: Very verbose output (not currently used)

### Examples

```bash
# Show informational logs (recommended for monitoring)
RUST_LOG=info ./target/release/music_search --api qq --query "test"

# Show debug logs (for troubleshooting)
RUST_LOG=debug ./target/release/music_search --api netease --query "search term"

# Filter logs by module
RUST_LOG=music_search_rs=debug ./target/release/music_search

# Combine with command-line arguments
RUST_LOG=info ./target/release/music_search --api qq --cookie "$MUSIC_COOKIE" --query "test"
```

### What Gets Logged

With `RUST_LOG=info`, you'll see:
- API client initialization
- Search requests and results
- Playlist/album fetching operations
- Lyrics retrieval status
- Success/failure of operations

With `RUST_LOG=debug`, you'll additionally see:
- HTTP request URLs and parameters
- Response sizes and status codes
- Cookie usage for authentication
- Detailed parsing information
- Decryption progress (for lyrics)

Example log output:
```
2024-01-15T10:30:45Z  INFO Initializing QQ Music API client
2024-01-15T10:30:45Z DEBUG No cookie provided, using anonymous access
2024-01-15T10:30:45Z  INFO Searching for 'test' with type SongId
2024-01-15T10:30:46Z DEBUG POST request to: https://c.y.qq.com/soso/fcgi-bin/client_search_cp
2024-01-15T10:30:46Z DEBUG Response received, length: 12543 bytes
2024-01-15T10:30:46Z  INFO Search successful, found 15 songs
```

### Default Behavior

When `RUST_LOG` is not set, the binary defaults to showing only warnings and errors to keep output clean during normal operation.

### Interactive Workflow

The binary provides an interactive command-line interface that guides you through the process:

1. **Select Music Service**
   ```
   Select music service:
   1. NetEase Cloud Music
   2. QQ Music
   Enter choice (1 or 2): 
   ```

2. **Enter Search Query**
   ```
   Enter song name or artist to search: 告白气球
   ```

3. **View Search Results**
   ```
   Search Results:
   No.  Song                                     Artist                         Album               
   ----------------------------------------------------------------------------------------------------
   1    告白气球                                   周杰伦                           周杰伦的床边故事        
   2    告白气球 (Live)                            周杰伦                           地表最强演唱会         
   ...
   ```

4. **Select a Song**
   ```
   Enter song number to download lyrics (or 0 to exit): 1
   ```

5. **Choose Lyrics Type**
   ```
   Available lyrics:
   1. Original
   2. Translation
   3. Transliteration
   
   Enter lyrics type number to download (or 0 to download all): 0
   ```

6. **Download Complete**
   ```
   ✓ Saved Original lyrics to: lyrics/告白气球 - 周杰伦_original.lrc
   ✓ Saved Translation lyrics to: lyrics/告白气球 - 周杰伦_translation.lrc
   ✓ Saved Transliteration lyrics to: lyrics/告白气球 - 周杰伦_transliteration.lrc
   
   ✓ Download complete!
   ```

## Features

### Supported Services
- **NetEase Cloud Music** - One of China's largest music streaming platforms
- **QQ Music** - Tencent's music streaming service

### Lyrics Types
- **Original** - The original lyrics in their native language
- **Translation** - Translated lyrics (when available)
- **Transliteration** - Romanized/phonetic lyrics (when available)

### Output Format
- Lyrics are saved in standard `.lrc` format
- Files are saved to the `lyrics/` directory (created automatically)
- Filenames are sanitized to be filesystem-safe
- Format: `{Song Title} - {Artist}_{lyrics_type}.lrc`

### Smart Features
- **Filename Sanitization** - Invalid characters are automatically replaced
- **Truncated Display** - Long song/artist/album names are truncated in the table view
- **Multiple Downloads** - Option to download all lyrics types at once (enter 0)
- **Error Handling** - Graceful handling of missing lyrics or network errors

## Examples

### Example 1: Interactive Mode (Default)

```bash
$ ./target/release/music_search
=== Music Search & Lyrics Downloader ===

Select music service:
1. NetEase Cloud Music
2. QQ Music
Enter choice (1 or 2): 2

Using QQ Music service

Enter song name or artist to search: 告白气球

Searching for '告白气球'...

Search Results:
No.  Song                                     Artist                         Album               
----------------------------------------------------------------------------------------------------
1    告白气球                                     周杰伦                            周杰伦的床边故事            
2    告白气球 (Live)                              周杰伦                            周杰伦地表最强世界巡回演唱会      

Enter song number to download lyrics (or 0 to exit): 1

Selected: 告白气球 - 周杰伦

Fetching lyrics...

Available lyrics:
1. Original

Enter lyrics type number to download (or 0 to download all): 1

✓ Saved Original lyrics to: lyrics/告白气球 - 周杰伦_original.lrc

✓ Download complete!
```

### Example 2: Command-Line Mode with QQ Music

```bash
$ ./target/release/music_search --api qq --query "告白气球"
=== Music Search & Lyrics Downloader ===


Using QQ Music service


Searching for '告白气球'...

Search Results:
No.  Song                                     Artist                         Album               
----------------------------------------------------------------------------------------------------
1    告白气球                                     周杰伦                            周杰伦的床边故事            
2    告白气球                                     二珂                                                 

Enter song number to download lyrics (or 0 to exit): 1

# ... (interactive lyrics selection continues)
```

### Example 3: Using NetEase with Cookie Authentication

```bash
$ ./target/release/music_search
=== Music Search & Lyrics Downloader ===

Select music service:
1. NetEase Cloud Music
2. QQ Music
Enter choice (1 or 2): 1

Using NetEase Cloud Music service

Enter song name or artist to search: 稻香

Searching for '稻香'...

Search Results:
No.  Song                                     Artist                         Album               
----------------------------------------------------------------------------------------------------
1    稻香                                      周杰伦                           魔杰座               
2    稻香 (Live)                               周杰伦                           2013 摩天伦世界巡回...   

Enter song number to download lyrics (or 0 to exit): 1

Selected: 稻香 - 周杰伦

Fetching lyrics...

Available lyrics:
1. Original

Enter lyrics type number to download (or 0 to download all): 1

✓ Saved Original lyrics to: lyrics/稻香 - 周杰伦_original.lrc

✓ Download complete!
```

### Example 2: Download All Lyrics Types

```bash
$ ./target/release/music_search
=== Music Search & Lyrics Downloader ===

Select music service:
1. NetEase Cloud Music
2. QQ Music
Enter choice (1 or 2): 2

Using QQ Music service

Enter song name or artist to search: 夜曲

Searching for '夜曲'...

Search Results:
No.  Song                                     Artist                         Album               
----------------------------------------------------------------------------------------------------
1    夜曲                                      周杰伦                           十一月的萧邦          

Enter song number to download lyrics (or 0 to exit): 1

Selected: 夜曲 - 周杰伦

Fetching lyrics...

Available lyrics:
1. Original
2. Translation

Enter lyrics type number to download (or 0 to download all): 0

✓ Saved Original lyrics to: lyrics/夜曲 - 周杰伦_original.lrc
✓ Saved Translation lyrics to: lyrics/夜曲 - 周杰伦_translation.lrc

✓ Download complete!
```

## Output Directory

All lyrics files are saved to the `lyrics/` directory in your current working directory:

```bash
$ ls -l lyrics/
-rw-r--r-- 1 user user 2147 Oct 25 14:05 告白气球 - 周杰伦_original.lrc
-rw-r--r-- 1 user user 1823 Oct 25 14:05 告白气球 - 周杰伦_translation.lrc
-rw-r--r-- 1 user user 2456 Oct 25 14:05 稻香 - 周杰伦_original.lrc
```

## Error Handling

The binary handles various error scenarios gracefully:

- **No results found** - Displays a message and exits
- **Network errors** - Returns error information from the API
- **No lyrics available** - Informs the user and exits
- **Invalid selections** - Prompts user to enter valid input

## Limitations

- Requires active internet connection
- Some songs may not have all lyrics types available
- Search results are limited to the first 10 results
- Cookie-based authentication not currently supported (some premium content may be unavailable)

## Tips

1. **Chinese Characters** - The tool works best with Chinese song/artist names
2. **Multiple Artists** - When multiple artists are involved, they're displayed as comma-separated
3. **Batch Downloads** - Use option 0 to download all available lyrics types at once
4. **File Organization** - The `lyrics/` directory is created automatically if it doesn't exist

## Troubleshooting

### "No results found"
- Try simplifying your search query
- Try searching with artist name instead of song title
- Switch to a different music service

### "No lyrics found for this song"
- The song may not have lyrics uploaded to the service
- Try the other music service (NetEase vs QQ Music)

### Network Errors
- Check your internet connection
- The service API may be temporarily unavailable
- Try again after a few moments

## Technical Details

- **Language**: Rust 2021 Edition
- **Async Runtime**: Tokio
- **HTTP Client**: reqwest
- **Output Format**: LRC (standard lyric format with timestamps)
- **Encoding**: UTF-8

## License

This binary is part of the music-search-rs project and follows the same license terms.
