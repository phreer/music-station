# Lyrics Fetching API

This document describes the internal APIs for fetching lyrics from various internet sources.

## Architecture

The lyrics fetching system is built on a trait-based architecture that allows easy extensibility:

```
┌─────────────────┐
│ LyricsAggregator│  ← Main entry point
└────────┬────────┘
         │
         ├──► Provider 1 (e.g., Genius)
         ├──► Provider 2 (e.g., Netease)
         ├──► Provider 3 (e.g., Local Files)
         └──► Provider N (Your Custom Provider)
```

## Core Traits and Types

### `LyricsProvider` Trait

The main trait that all lyrics providers must implement:

```rust
#[async_trait]
pub trait LyricsProvider: Send + Sync {
    fn name(&self) -> &str;
    fn supports_synced(&self) -> bool { false }
    fn requires_auth(&self) -> bool { false }
    
    async fn search(&self, query: &LyricsQuery) -> Result<Vec<LyricsSearchResult>>;
    async fn fetch(&self, result_id: &str) -> Result<LyricsResponse>;
    async fn search_and_fetch(&self, query: &LyricsQuery) -> Result<Option<LyricsResponse>>;
    async fn health_check(&self) -> Result<bool>;
}
```

### Key Types

- **`LyricsQuery`**: Search parameters (title, artist, album, duration)
- **`LyricsSearchResult`**: A single search result with confidence score
- **`LyricsResponse`**: Complete lyrics with metadata
- **`LyricFormat`**: Either `Plain` or `Lrc` (synchronized)

## Usage Examples

### Basic Usage

```rust
use music_station::lyrics::fetcher::*;
use music_station::lyrics::providers::*;

// Create a provider
let provider = MockLyricsProvider::new("test")
    .add_lyrics("Song Title", "Artist Name", "lyrics content", LyricFormat::Plain);

// Search for lyrics
let query = LyricsQuery::new("Song Title")
    .with_artist("Artist Name");

let results = provider.search(&query).await?;
if let Some(result) = results.first() {
    let lyrics = provider.fetch(&result.id).await?;
    println!("Found lyrics: {}", lyrics.content);
}
```

### Using the Aggregator (Recommended)

The aggregator tries multiple providers with automatic fallback:

```rust
use music_station::lyrics::fetcher::*;
use music_station::lyrics::providers::*;

// Create aggregator with multiple providers
let aggregator = LyricsAggregator::new()
    .add_provider(Box::new(LocalLyricsProvider::new("/path/to/lyrics")))
    .add_provider(Box::new(MockLyricsProvider::new("fallback")))
    .add_provider(Box::new(YourCustomProvider::new()));

// Fetch lyrics (tries providers in order)
let query = LyricsQuery::new("Song Title").with_artist("Artist");
if let Some(lyrics) = aggregator.fetch_lyrics(&query).await? {
    println!("Source: {}", lyrics.source);
    println!("Format: {:?}", lyrics.format);
    println!("Content: {}", lyrics.content);
}
```

### Parallel Search

Search all providers simultaneously:

```rust
let results = aggregator.search_all(&query).await;

for (provider_name, result) in results {
    match result {
        Ok(matches) => {
            println!("{}: found {} results", provider_name, matches.len());
            for m in matches {
                println!("  - {} by {} (confidence: {:.2})", 
                    m.title, m.artist, m.confidence);
            }
        }
        Err(e) => println!("{}: error - {:?}", provider_name, e),
    }
}
```

## Implementing a Custom Provider

Here's a template for creating your own lyrics provider:

```rust
use music_station::lyrics::fetcher::*;
use anyhow::Result;
use async_trait::async_trait;

pub struct MyCustomProvider {
    api_key: String,
    client: reqwest::Client,
}

impl MyCustomProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LyricsProvider for MyCustomProvider {
    fn name(&self) -> &str {
        "my-custom-provider"
    }

    fn supports_synced(&self) -> bool {
        true  // If your provider supports LRC format
    }

    fn requires_auth(&self) -> bool {
        true  // If your provider needs API key
    }

    async fn search(&self, query: &LyricsQuery) -> Result<Vec<LyricsSearchResult>> {
        // 1. Build API request with query parameters
        let url = format!("https://api.example.com/search?q={}&artist={}", 
            query.title, 
            query.artist.as_deref().unwrap_or("")
        );

        // 2. Make HTTP request
        let response = self.client
            .get(&url)
            .header("Authorization", &self.api_key)
            .send()
            .await?;

        // 3. Parse response
        let data: Vec<ApiSearchResult> = response.json().await?;

        // 4. Convert to LyricsSearchResult
        Ok(data.into_iter().map(|item| LyricsSearchResult {
            id: item.id,
            title: item.title,
            artist: item.artist,
            album: item.album,
            duration: None,
            confidence: calculate_confidence(&query, &item),
        }).collect())
    }

    async fn fetch(&self, result_id: &str) -> Result<LyricsResponse> {
        // 1. Fetch lyrics content from API
        let url = format!("https://api.example.com/lyrics/{}", result_id);
        let response = self.client
            .get(&url)
            .header("Authorization", &self.api_key)
            .send()
            .await?;

        let data: ApiLyricsData = response.json().await?;

        // 2. Determine format
        let format = if data.is_synced {
            LyricFormat::Lrc
        } else {
            LyricFormat::Plain
        };

        // 3. Return LyricsResponse
        Ok(LyricsResponse {
            content: data.lyrics,
            format,
            language: data.language,
            source: self.name().to_string(),
            url: Some(data.url),
            metadata: LyricsMetadata {
                contributor: data.contributor,
                source_updated_at: data.updated_at,
                copyright: data.copyright,
                notes: None,
            },
        })
    }
}

// Helper function to calculate match confidence
fn calculate_confidence(query: &LyricsQuery, result: &ApiSearchResult) -> f32 {
    let mut score = 0.0;
    
    // Exact title match
    if query.title.to_lowercase() == result.title.to_lowercase() {
        score += 0.5;
    } else if result.title.to_lowercase().contains(&query.title.to_lowercase()) {
        score += 0.3;
    }
    
    // Artist match
    if let Some(query_artist) = &query.artist {
        if query_artist.to_lowercase() == result.artist.to_lowercase() {
            score += 0.3;
        }
    }
    
    // Duration match (within 5 seconds)
    if let (Some(query_duration), Some(result_duration)) = (query.duration, result.duration) {
        let diff = (query_duration.as_secs() as i64 - result_duration.as_secs() as i64).abs();
        if diff <= 5 {
            score += 0.2;
        }
    }
    
    score.min(1.0)
}
```

## Provider Examples

### 1. Local File Provider

Reads lyrics from local filesystem:

```rust
let local = LocalLyricsProvider::new("/home/user/lyrics");
```

Directory structure:
```
lyrics/
  ├── Artist 1/
  │   ├── Song 1.lrc
  │   └── Song 2.txt
  └── Artist 2/
      └── Song 3.lrc
```

### 2. Mock Provider

In-memory provider for testing:

```rust
let mock = MockLyricsProvider::new("test")
    .add_lyrics("Title", "Artist", "content...", LyricFormat::Plain)
    .add_lyrics("Title2", "Artist", "[00:00]content", LyricFormat::Lrc);
```

### 3. HTTP-based Provider Template

```rust
pub struct HttpLyricsProvider {
    base_url: String,
    client: reqwest::Client,
}

impl HttpLyricsProvider {
    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}/{}", self.base_url, path);
        Ok(self.client.get(&url).send().await?.json().await?)
    }
}
```

## Integration with Database

To save fetched lyrics to the database:

```rust
use music_station::lyrics::{LyricDatabase, fetcher::*};

// Fetch lyrics
let aggregator = setup_aggregator();
let query = LyricsQuery::new("Song").with_artist("Artist");
let lyrics = aggregator.fetch_lyrics(&query).await?;

// Save to database
if let Some(lyrics) = lyrics {
    let db = LyricDatabase::new("lyrics.db").await?;
    db.save_lyric(
        "track-id-here",
        lyrics.content,
        lyrics.format,
        lyrics.language,
        Some(lyrics.source),
    ).await?;
}
```

## Error Handling

All provider methods return `Result<T>`, allowing for graceful error handling:

```rust
match provider.search(&query).await {
    Ok(results) => { /* handle results */ }
    Err(e) => {
        tracing::error!("Provider {} failed: {:?}", provider.name(), e);
        // Try next provider or show error to user
    }
}
```

The aggregator automatically handles errors and tries the next provider.

## Best Practices

1. **Use confidence scores**: Only fetch lyrics with confidence > 0.5
2. **Implement health checks**: Verify provider availability before using
3. **Add rate limiting**: Respect API rate limits in your provider
4. **Cache responses**: Consider caching to reduce API calls
5. **Handle errors gracefully**: Don't panic - return errors and let aggregator handle fallback
6. **Log appropriately**: Use `tracing` for debugging and monitoring
7. **Test thoroughly**: Write unit tests for your provider

## Future Providers

Potential providers to implement:

- **Genius API**: English lyrics with annotations
- **Netease Music API**: Chinese lyrics (often with LRC)
- **Musixmatch**: Multi-language lyrics
- **AZLyrics**: Web scraping (check terms of service)
- **LyricWiki/LyricsGenius**: Community-contributed lyrics
- **Spotify API**: Limited lyrics access
- **Local LRC files**: Scan music library folders

## Testing

Run the test suite:

```bash
cargo test --package music-station --lib lyrics::fetcher
cargo test --package music-station --lib lyrics::providers
```

Create test fixtures:

```rust
#[tokio::test]
async fn test_my_provider() {
    let provider = MyProvider::new();
    let query = LyricsQuery::new("Test Song");
    
    let results = provider.search(&query).await.unwrap();
    assert!(!results.is_empty());
    
    let lyrics = provider.fetch(&results[0].id).await.unwrap();
    assert!(!lyrics.content.is_empty());
}
```
