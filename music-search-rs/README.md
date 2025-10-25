# Music Search RS

Rust implementation of music search services for NetEase Cloud Music (网易云音乐) and QQ Music (QQ音乐).

This is a Rust port of the music search functionality from the [163MusicLyrics](https://github.com/jitwxs/163MusicLyrics) project, providing a clean async API for searching songs, albums, and playlists, as well as fetching lyrics from both music services.

## Quick Start

### Using the CLI Binary

```bash
# Build and run the interactive tool
cargo build --bin music_search --release
./target/release/music_search

# Or use command-line arguments for automation
./target/release/music_search --api qq --query "告白气球"

# With cookie authentication for NetEase
export MUSIC_COOKIE="your_cookie_here"
./target/release/music_search --api netease --query "周杰伦"
```

See [BINARY_USAGE.md](./BINARY_USAGE.md) for detailed CLI documentation.

### Using as a Library

```rust
use music_search_rs::{NetEaseMusicApi, MusicApi, SearchType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = NetEaseMusicApi::new(None)?;
    let result = api.search("告白气球", SearchType::SongId).await?;
    Ok(())
}
```

## Features

- 🎵 **NetEase Cloud Music API**
  - Search songs, albums, and playlists
  - Fetch song details and lyrics (with translation and transliteration support)
  - AES + RSA encryption for API requests
  
- 🎶 **QQ Music API**
  - Search songs, albums, and playlists
  - Fetch song details and lyrics
  - Triple-DES decryption for encrypted lyrics
  
- 🔒 **Security**
  - Proper encryption implementation matching official APIs
  - Cookie support for authenticated requests

- ⚡ **Performance**
  - Async/await with tokio runtime
  - Efficient HTTP client with connection pooling

- 🖥️ **CLI Binary**
  - Interactive command-line tool for searching and downloading lyrics
  - Support for both NetEase and QQ Music
  - Download original, translated, and transliteration lyrics

## Installation

### As a Library

Add this to your `Cargo.toml`:

```toml
[dependencies]
music-search-rs = { path = "../music-search-rs" }
tokio = { version = "1.0", features = ["full"] }
```

### As a Binary

Build and install the CLI tool:

```bash
# Build release binary
cargo build --bin music_search --release

# Install to ~/.cargo/bin
cargo install --path . --bin music_search

# Or copy manually
sudo cp target/release/music_search /usr/local/bin/
```

The binary provides an interactive interface for searching songs and downloading lyrics. See [BINARY_USAGE.md](./BINARY_USAGE.md) for detailed usage instructions.

## Usage

### NetEase Cloud Music

```rust
use music_search_rs::{NetEaseMusicApi, MusicApi, SearchType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create API instance (optionally with cookie)
    let api = NetEaseMusicApi::new(None)?;
    
    // Search for songs
    let result = api.search("告白气球", SearchType::SongId).await?;
    
    if result.is_success() {
        if let Some(search_result) = result.data {
            for song in search_result.song_vos {
                println!("Song: {} - {}", song.title, song.author_name.join(", "));
            }
        }
    }
    
    // Get lyrics
    let lyric_result = api.get_lyric("", "186016", false).await?;
    if let Some(lyric) = lyric_result.data {
        println!("Lyrics: {}", lyric.lyric.unwrap_or_default());
    }
    
    Ok(())
}
```

### QQ Music

```rust
use music_search_rs::{QQMusicApi, MusicApi, SearchType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create API instance
    let api = QQMusicApi::new(None)?;
    
    // Search for albums
    let result = api.search("周杰伦", SearchType::AlbumId).await?;
    
    if result.is_success() {
        if let Some(search_result) = result.data {
            for album in search_result.album_vos {
                println!("Album: {} by {}", 
                    album.album_name, 
                    album.author_name.join(", ")
                );
            }
        }
    }
    
    Ok(())
}
```

### Using the Unified MusicApi Trait

```rust
use music_search_rs::{MusicApi, NetEaseMusicApi, QQMusicApi, SearchType};

async fn search_music(api: &dyn MusicApi, keyword: &str) -> Result<(), Box<dyn std::error::Error>> {
    let result = api.search(keyword, SearchType::SongId).await?;
    
    if let Some(search_result) = result.data {
        println!("Found {} songs from {:?}", 
            search_result.song_vos.len(),
            api.source()
        );
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let netease = NetEaseMusicApi::new(None)?;
    let qq = QQMusicApi::new(None)?;
    
    search_music(&netease, "告白气球").await?;
    search_music(&qq, "告白气球").await?;
    
    Ok(())
}
```

## API Reference

### MusicApi Trait

The `MusicApi` trait provides a unified interface for both music services:

- `source()` - Get the search source (NetEase or QQ Music)
- `search(keyword, search_type)` - Search for songs, albums, or playlists
- `get_playlist(playlist_id)` - Get playlist information
- `get_album(album_id)` - Get album information
- `get_songs(song_ids)` - Get multiple songs information
- `get_song_link(song_id)` - Get song URL
- `get_lyric(id, display_id, is_verbatim)` - Get lyric information

### Search Types

```rust
pub enum SearchType {
    SongId,      // Search for songs
    AlbumId,     // Search for albums
    PlaylistId,  // Search for playlists
}
```

### Search Sources

```rust
pub enum SearchSource {
    NetEaseMusic,  // NetEase Cloud Music
    QQMusic,       // QQ Music
}
```

## Architecture

```
music-search-rs/
├── src/
│   ├── lib.rs           # Main library with MusicApi trait
│   ├── error.rs         # Error types
│   ├── models.rs        # Common data models
│   ├── netease/         # NetEase Cloud Music implementation
│   │   ├── mod.rs
│   │   ├── api.rs       # API implementation
│   │   └── models.rs    # NetEase-specific models
│   └── qqmusic/         # QQ Music implementation
│       ├── mod.rs
│       ├── api.rs       # API implementation
│       ├── models.rs    # QQ Music-specific models
│       └── decrypt.rs   # Triple-DES lyric decryption
└── Cargo.toml
```

## Encryption Details

### NetEase Cloud Music

NetEase uses a two-stage AES encryption with RSA for the secret key:

1. Generate a random 16-character secret key
2. AES-128-CBC encrypt the payload twice (first with NONCE, then with secret key)
3. RSA encrypt the secret key with public key
4. Send encrypted params and encrypted key to API

### QQ Music

QQ Music uses Triple-DES encryption for lyrics:

1. Hex decode the encrypted lyric string
2. Triple-DES decrypt with a fixed key
3. Zlib decompress the decrypted data
4. Parse XML to extract lyrics

## Testing

Run tests with:

```bash
cd cross-platform/music-search-rs
cargo test
```

## Dependencies

- `reqwest` - HTTP client
- `tokio` - Async runtime
- `serde` / `serde_json` - Serialization
- `aes` / `cbc` / `rsa` - Encryption (NetEase)
- `flate2` - Compression (QQ Music lyrics)
- `base64` / `hex` - Encoding

## License

MIT License - Same as the parent project [163MusicLyrics](https://github.com/jitwxs/163MusicLyrics)

## Credits

This is a Rust port of the C# implementation from [163MusicLyrics](https://github.com/jitwxs/163MusicLyrics) by [@jitwxs](https://github.com/jitwxs).

## Disclaimer

This library is for educational purposes only. Please respect the terms of service of NetEase Cloud Music and QQ Music.
