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

### Playlist Management

#### List All Playlists
```bash
cargo run --bin music-client -- playlist list
```

#### Create a New Playlist
```bash
# With name only
cargo run --bin music-client -- playlist create "My Favorites"

# With name and description
cargo run --bin music-client -- playlist create "Chill Vibes" --description "Relaxing music"
```

#### Get Playlist Details
```bash
cargo run --bin music-client -- playlist info <playlist-id>
```
This shows the playlist name, description, creation date, and all tracks in the playlist.

#### Update a Playlist
```bash
# Update name
cargo run --bin music-client -- playlist update <playlist-id> --name "New Name"

# Update description
cargo run --bin music-client -- playlist update <playlist-id> --description "New description"

# Update both
cargo run --bin music-client -- playlist update <playlist-id> --name "New Name" --description "New desc"
```

#### Delete a Playlist
```bash
cargo run --bin music-client -- playlist delete <playlist-id>
```

#### Add Tracks to Playlist
```bash
# Add one track
cargo run --bin music-client -- playlist add-track <playlist-id> <track-id>

# Add multiple tracks
cargo run --bin music-client -- playlist add-track <playlist-id> <track-id-1> <track-id-2> <track-id-3>
```

#### Remove Tracks from Playlist
```bash
# Remove one track
cargo run --bin music-client -- playlist remove-track <playlist-id> <track-id>

# Remove multiple tracks
cargo run --bin music-client -- playlist remove-track <playlist-id> <track-id-1> <track-id-2>
```

#### Play a Playlist
```bash
cargo run --bin music-client -- playlist play <playlist-id>
```
This will play all tracks in the playlist in order.

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

# Create a playlist
cargo run --bin music-client -- playlist create "My Favorites" --description "Best songs"
# Output: ✓ Playlist created successfully!
#         ID: 550e8400-e29b-41d4-a716-446655440000
#         Name: My Favorites
#         Description: Best songs

# Add tracks to playlist
cargo run --bin music-client -- playlist add-track 550e8400-e29b-41d4-a716-446655440000 a1b2c3d4e5f6 x9y8z7w6v5u4

# List playlists
cargo run --bin music-client -- playlist list
# Output: Playlists (1):
#         📋 My Favorites
#            Description: Best songs
#            Tracks: 2
#            ID: 550e8400-e29b-41d4-a716-446655440000

# View playlist details
cargo run --bin music-client -- playlist info 550e8400-e29b-41d4-a716-446655440000

# Play the playlist
cargo run --bin music-client -- playlist play 550e8400-e29b-41d4-a716-446655440000
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
