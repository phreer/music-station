# Debug Logging Guide

## Overview

Music Station now includes comprehensive debug logging for monitoring server activity, troubleshooting issues, and understanding request flows.

## Log Levels

The server uses the following log levels:

- **DEBUG**: Detailed information about requests and operations
- **INFO**: General informational messages (startup, configuration)
- **WARN**: Warning messages (track not found, etc.)
- **ERROR**: Error messages (file I/O failures, update failures)

## Configuration

Debug logging is enabled by default with these settings:

- **Max Level**: DEBUG (shows all log messages)
- **Target**: Enabled (shows module path)
- **Thread IDs**: Enabled (shows which thread handled request)
- **File & Line**: Enabled (shows source location)

## What's Logged

### Server Startup
```
INFO  Starting Music Station
INFO  Library path: /path/to/music
INFO  Scanning library at: /path/to/music
INFO  Found track: Song Title
INFO  Scan complete. Found 42 tracks
INFO  Server listening on http://0.0.0.0:3000
```

### HTTP Requests (TraceLayer)
```
DEBUG request started method=GET uri=/tracks
DEBUG request finished method=GET uri=/tracks status=200 duration=5ms
```

### API Handlers

**List Tracks:**
```
DEBUG Fetching all tracks
DEBUG Returning 42 tracks
```

**Get Track:**
```
DEBUG Fetching track with id: a1b2c3d4
DEBUG Track a1b2c3d4 found
```

**Stream Track:**
```
DEBUG Streaming track with id: a1b2c3d4
DEBUG Streaming file: /path/to/music/song.flac
DEBUG Streaming 15728640 bytes for track a1b2c3d4
```

**Update Track:**
```
DEBUG Updating track a1b2c3d4 with metadata: title=Some("New Title"), artist=Some("New Artist"), album=None
INFO  Updated metadata for track: New Title (a1b2c3d4)
DEBUG Successfully updated track a1b2c3d4
```

### Errors
```
WARN  Track a1b2c3d4 not found
ERROR Failed to update track metadata: Failed to write metadata to file
```

## Viewing Logs

### Development
When running with `cargo run`, logs are printed to stdout with colored output:

```bash
cargo run -- --library /path/to/music
```

### Production
For production deployments, redirect logs to a file:

```bash
./target/release/music-station --library /path/to/music 2>&1 | tee server.log
```

### Filtering Logs

Set the `RUST_LOG` environment variable to filter log output:

```bash
# Show only INFO and above (less verbose)
RUST_LOG=info cargo run -- --library /path/to/music

# Show only music_station logs
RUST_LOG=music_station=debug cargo run -- --library /path/to/music

# Show all DEBUG logs including dependencies
RUST_LOG=debug cargo run -- --library /path/to/music

# Show only ERROR logs
RUST_LOG=error cargo run -- --library /path/to/music
```

## Log Format

Each log line includes:

```
LEVEL [target] thread_id file:line - message
```

Example:
```
DEBUG [music_station::server] ThreadId(2) src/server.rs:42 - Fetching track with id: a1b2c3d4
```

## Troubleshooting with Logs

### Track Not Loading
Look for:
```
WARN  Failed to parse /path/to/file.flac: <error details>
```

### API Request Issues
Look for:
```
DEBUG request started method=GET uri=/tracks/:id
WARN  Track <id> not found
DEBUG request finished status=404
```

### Metadata Update Failures
Look for:
```
DEBUG Updating track <id> with metadata: ...
ERROR Failed to update track metadata: <error details>
```

### File Streaming Issues
Look for:
```
DEBUG Streaming track with id: <id>
DEBUG Streaming file: <path>
ERROR Failed to read file: <error details>
```

## Performance Monitoring

The TraceLayer logs request duration:

```
DEBUG request finished method=GET uri=/stream/a1b2c3d4 status=200 duration=125ms
```

Use this to identify slow requests and optimize performance.

## Disabling Debug Logs

To disable debug logs (e.g., in production):

### Option 1: Environment Variable
```bash
RUST_LOG=info cargo run -- --library /path/to/music
```

### Option 2: Modify Code
Edit `src/main.rs`:

```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)  // Change from DEBUG to INFO
    .with_target(true)
    .with_thread_ids(true)
    .with_file(true)
    .with_line_number(true)
    .init();
```

## Advanced Configuration

### JSON Logging
For structured logging (useful for log aggregation):

```rust
tracing_subscriber::fmt()
    .json()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

### Custom Format
To customize log format, modify the subscriber in `src/main.rs`:

```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_target(false)      // Hide module path
    .with_thread_ids(false)  // Hide thread IDs
    .with_file(false)        // Hide file location
    .with_line_number(false) // Hide line numbers
    .compact()               // Use compact format
    .init();
```

## Best Practices

1. **Development**: Keep DEBUG level enabled for detailed diagnostics
2. **Production**: Use INFO level for normal operations
3. **Debugging Issues**: Temporarily enable DEBUG for specific modules
4. **Performance**: Disable thread IDs and source locations in production
5. **Monitoring**: Parse logs for metrics and alerts

## Example Session

```bash
# Start server with debug logging
$ cargo run -- --library ~/Music/FLAC

 INFO  music_station: Starting Music Station
 INFO  music_station: Library path: /Users/user/Music/FLAC
 INFO  music_station::library: Scanning library at: /Users/user/Music/FLAC
 INFO  music_station::library: Found track: Awesome Song
 INFO  music_station::library: Found track: Another Track
 INFO  music_station::library: Scan complete. Found 2 tracks
 INFO  music_station: Server listening on http://0.0.0.0:3000
 INFO  music_station: Web Client:
 INFO  music_station:   http://localhost:3000/web/index.html

# User accesses web client
DEBUG tower_http::trace: request started method=GET uri=/web/index.html
DEBUG tower_http::trace: request finished status=200 duration=2ms

# User loads track list
DEBUG tower_http::trace: request started method=GET uri=/tracks
DEBUG music_station::server: Fetching all tracks
DEBUG music_station::server: Returning 2 tracks
DEBUG tower_http::trace: request finished status=200 duration=1ms

# User edits a track
DEBUG tower_http::trace: request started method=PUT uri=/tracks/a1b2c3d4
DEBUG music_station::server: Updating track a1b2c3d4 with metadata: title=Some("New Title"), artist=None, album=None
 INFO  music_station::library: Updated metadata for track: New Title (a1b2c3d4)
DEBUG music_station::server: Successfully updated track a1b2c3d4
DEBUG tower_http::trace: request finished status=200 duration=45ms
```
