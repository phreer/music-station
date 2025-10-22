//! Example lyrics provider implementations

use super::fetcher::*;
use super::LyricFormat;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// A mock/example lyrics provider for testing and demonstration
/// 
/// This provider stores lyrics in memory and can be used for:
/// - Testing the lyrics fetching infrastructure
/// - Providing fallback lyrics
/// - Demonstrating how to implement the LyricsProvider trait
pub struct MockLyricsProvider {
    name: String,
    lyrics_db: HashMap<String, (String, LyricFormat)>,
}

impl MockLyricsProvider {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            lyrics_db: HashMap::new(),
        }
    }

    /// Add a lyrics entry to this mock provider
    pub fn add_lyrics(
        mut self,
        title: impl Into<String>,
        artist: impl Into<String>,
        content: String,
        format: LyricFormat,
    ) -> Self {
        let key = format!("{}:{}", title.into().to_lowercase(), artist.into().to_lowercase());
        self.lyrics_db.insert(key, (content, format));
        self
    }

    /// Calculate string similarity (simple implementation)
    fn similarity(a: &str, b: &str) -> f32 {
        let a = a.to_lowercase();
        let b = b.to_lowercase();

        if a == b {
            return 1.0;
        }

        if a.contains(&b) || b.contains(&a) {
            return 0.8;
        }

        // Simple Levenshtein-inspired similarity
        let len_a = a.len() as f32;
        let len_b = b.len() as f32;
        let len_diff = (len_a - len_b).abs();
        let max_len = len_a.max(len_b);

        if max_len == 0.0 {
            return 0.0;
        }

        (1.0 - len_diff / max_len) * 0.5
    }
}

#[async_trait]
impl LyricsProvider for MockLyricsProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn supports_synced(&self) -> bool {
        true // Can provide both plain and LRC
    }

    async fn search(&self, query: &LyricsQuery) -> Result<Vec<LyricsSearchResult>> {
        let mut results = Vec::new();

        for (key, _) in &self.lyrics_db {
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() != 2 {
                continue;
            }

            let title = parts[0];
            let artist = parts[1];

            // Calculate match confidence
            let title_similarity = Self::similarity(&query.title, title);
            let artist_similarity = if let Some(query_artist) = &query.artist {
                Self::similarity(query_artist, artist)
            } else {
                0.5 // Neutral if no artist specified
            };

            let confidence = (title_similarity * 0.7 + artist_similarity * 0.3).min(1.0);

            if confidence > 0.3 {
                results.push(LyricsSearchResult {
                    id: key.clone(),
                    title: title.to_string(),
                    artist: artist.to_string(),
                    album: query.album.clone(),
                    duration: query.duration,
                    confidence,
                });
            }
        }

        // Sort by confidence descending
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(results)
    }

    async fn fetch(&self, result_id: &str) -> Result<LyricsResponse> {
        let (content, format) = self
            .lyrics_db
            .get(result_id)
            .ok_or_else(|| anyhow::anyhow!("Lyrics not found: {}", result_id))?;

        Ok(LyricsResponse {
            content: content.clone(),
            format: format.clone(),
            language: Some("en".to_string()),
            source: self.name.clone(),
            url: None,
            metadata: LyricsMetadata {
                contributor: Some("Mock Provider".to_string()),
                source_updated_at: None,
                copyright: None,
                notes: Some("Example lyrics for testing".to_string()),
            },
        })
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true) // Always healthy for mock provider
    }
}

/// A local file-based lyrics provider
/// 
/// Reads lyrics from a local directory structure:
/// ```
/// lyrics/
///   ├── artist1/
///   │   ├── song1.lrc
///   │   └── song2.txt
///   └── artist2/
///       └── song3.lrc
/// ```
pub struct LocalLyricsProvider {
    root_path: std::path::PathBuf,
}

impl LocalLyricsProvider {
    pub fn new(root_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            root_path: root_path.into(),
        }
    }

    /// Scan the directory structure and find matching files
    async fn scan_for_matches(&self, query: &LyricsQuery) -> Result<Vec<LyricsSearchResult>> {
        use tokio::fs;

        let mut results = Vec::new();

        if !self.root_path.exists() {
            return Ok(results);
        }

        let mut entries = fs::read_dir(&self.root_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {
                // Assume directory name is artist
                let artist = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                let mut song_entries = fs::read_dir(&path).await?;

                while let Some(song_entry) = song_entries.next_entry().await? {
                    let song_path = song_entry.path();
                    let extension = song_path.extension().and_then(|e| e.to_str());

                    if extension == Some("lrc") || extension == Some("txt") {
                        let title = song_path
                            .file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();

                        // Simple matching
                        let title_match = title.to_lowercase().contains(&query.title.to_lowercase());
                        let artist_match = query.artist.as_ref()
                            .map(|a| artist.to_lowercase().contains(&a.to_lowercase()))
                            .unwrap_or(true);

                        if title_match && artist_match {
                            results.push(LyricsSearchResult {
                                id: song_path.to_string_lossy().to_string(),
                                title,
                                artist: artist.clone(),
                                album: None,
                                duration: None,
                                confidence: if title_match && artist_match { 0.9 } else { 0.6 },
                            });
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}

#[async_trait]
impl LyricsProvider for LocalLyricsProvider {
    fn name(&self) -> &str {
        "local"
    }

    fn supports_synced(&self) -> bool {
        true // Can read both .lrc and .txt files
    }

    async fn search(&self, query: &LyricsQuery) -> Result<Vec<LyricsSearchResult>> {
        self.scan_for_matches(query).await
    }

    async fn fetch(&self, result_id: &str) -> Result<LyricsResponse> {
        use tokio::fs;
        use std::path::Path;

        let path = Path::new(result_id);
        let content = fs::read_to_string(path).await?;

        let format = if path.extension().and_then(|e| e.to_str()) == Some("lrc") {
            LyricFormat::Lrc
        } else {
            LyricFormat::Plain
        };

        Ok(LyricsResponse {
            content,
            format,
            language: None,
            source: "local".to_string(),
            url: None,
            metadata: LyricsMetadata {
                contributor: Some("Local file".to_string()),
                source_updated_at: None,
                copyright: None,
                notes: Some(format!("From: {}", result_id)),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockLyricsProvider::new("test-mock")
            .add_lyrics(
                "Test Song",
                "Test Artist",
                "Line 1\nLine 2\nLine 3".to_string(),
                LyricFormat::Plain,
            )
            .add_lyrics(
                "Another Song",
                "Test Artist",
                "[00:00.00]Line 1\n[00:05.00]Line 2".to_string(),
                LyricFormat::Lrc,
            );

        // Test search
        let query = LyricsQuery::new("Test Song").with_artist("Test Artist");
        let results = provider.search(&query).await.unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].title, "test song");
        assert!(results[0].confidence > 0.8);

        // Test fetch
        let lyrics = provider.fetch(&results[0].id).await.unwrap();
        assert_eq!(lyrics.format, LyricFormat::Plain);
        assert!(lyrics.content.contains("Line 1"));
    }

    #[tokio::test]
    async fn test_similarity() {
        assert_eq!(MockLyricsProvider::similarity("test", "test"), 1.0);
        assert!(MockLyricsProvider::similarity("test song", "test") > 0.7);
        assert!(MockLyricsProvider::similarity("hello", "world") < 0.5);
    }
}
