# Music Client Usage Guide

## Quick Start

### 1. Start the Server
```bash
# Start server with your music library
cargo run -- --library /path/to/your/music/folder

# Or use environment variable
export MUSIC_LIBRARY_PATH=/path/to/your/music
cargo run
```

### 2. Use the Client

#### List All Tracks
```bash
cargo run --bin music-client
# or explicitly
cargo run --bin music-client -- list
```

#### Get Track Details
```bash
cargo run --bin music-client -- info <track-id>
```
Copy the track ID from the list output.

#### Play a Single Track
```bash
cargo run --bin music-client -- play <track-id>
```
This will:
- Display track information
- Stream the audio from the server
- Play it through your default audio output
- Block until playback finishes

#### Play All Tracks
```bash
cargo run --bin music-client -- play-all
```
This will queue up all tracks and play them sequentially.

### 3. Custom Server URL
If your server is running on a different host/port:
```bash
cargo run --bin music-client -- --server http://192.168.1.100:8080 list
cargo run --bin music-client -- --server http://192.168.1.100:8080 play <track-id>
```

## Examples

```bash
# Terminal 1: Start server
cd /Users/phree/workspace/music-station
cargo run -- --library ~/Music/FLAC

# Terminal 2: Browse and play
cd /Users/phree/workspace/music-station

# List tracks
cargo run --bin music-client

# Output example:
# Music Library (3 tracks):
# --------------------------------------------------------------------------------
# 1. Artist Name - Song Title
#    Album: Album Name
#    Duration: 03:45
#    File: /Users/phree/Music/FLAC/song.flac
#    ID: a1b2c3d4e5f6...
#    Stream: http://localhost:3000/stream/a1b2c3d4e5f6...

# Play track by ID
cargo run --bin music-client -- play a1b2c3d4e5f6

# Play all tracks
cargo run --bin music-client -- play-all
```

## Features

### Playback Controls
- **Stop playback**: Press `Ctrl+C` in the client terminal
- **Track info**: Displays artist, title, album, and duration before playing
- **Progress**: Shows playback status messages

### Audio Format Support
Currently supports FLAC only. The client uses the Rodio library which supports:
- FLAC (currently enabled)
- MP3, WAV, Vorbis, etc. (can be added later)

## Troubleshooting

### "Failed to initialize audio output"
- Check that your system has a working audio output device
- On macOS: Check System Preferences > Sound > Output
- Try running: `cargo run --bin music-client -- play <track-id>` with verbose output

### "Track not found"
- Verify the track ID is correct (copy from `list` output)
- Ensure the server is running
- Check the server terminal for any errors

### "Failed to connect to server"
- Verify server is running on the expected URL
- Default is `http://localhost:3000`
- Use `--server` flag to specify custom URL

### No audio plays
- Check volume levels on your system
- Verify the FLAC file is valid
- Try playing a different track
- Check server logs for streaming errors

## Technical Notes

- Audio is fully downloaded before playback starts (no streaming playback)
- Uses system default audio output device
- Blocks terminal until playback completes
- Each track in `play-all` is queued to the same audio sink
