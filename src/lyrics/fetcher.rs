//! Lyrics fetching infrastructure
//!
//! This module provides a trait-based system for fetching lyrics from various internet sources.
//! It supports both synchronized (LRC format) and plain text lyrics.

use super::{LyricFormat, Lyric};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Search query for finding lyrics online
#[derive(Debug, Clone)]
pub struct LyricsQuery {
    /// Track title (required)
    pub title: String,
    /// Artist name (optional but recommended for better matching)
    pub artist: Option<String>,
    /// Album name (optional)
    pub album: Option<String>,
    /// Track duration in seconds (helps with matching accuracy)
    pub duration: Option<Duration>,
}

impl LyricsQuery {
    /// Create a new lyrics query with just title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            artist: None,
            album: None,
            duration: None,
        }
    }

    /// Set the artist name
    pub fn with_artist(mut self, artist: impl Into<String>) -> Self {
        self.artist = Some(artist.into());
        self
    }

    /// Set the album name
    pub fn with_album(mut self, album: impl Into<String>) -> Self {
        self.album = Some(album.into());
        self
    }

    /// Set the track duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }
}

/// Search result from a lyrics provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsSearchResult {
    /// Result ID (provider-specific identifier)
    pub id: String,
    /// Track title
    pub title: String,
    /// Artist name
    pub artist: String,
    /// Album name (if available)
    pub album: Option<String>,
    /// Track duration (if available)
    pub duration: Option<Duration>,
    /// Match confidence score (0.0 to 1.0)
    pub confidence: f32,
}

/// Complete lyrics response from a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsResponse {
    /// The lyrics content
    pub content: String,
    /// Lyrics format (plain or LRC)
    pub format: LyricFormat,
    /// Language code (e.g., "en", "zh", "ja")
    pub language: Option<String>,
    /// Source provider name
    pub source: String,
    /// Original lyrics URL (if available)
    pub url: Option<String>,
    /// Additional metadata
    pub metadata: LyricsMetadata,
}

/// Additional metadata about the lyrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LyricsMetadata {
    /// Contributor or translator information
    pub contributor: Option<String>,
    /// When the lyrics were added/updated at the source
    pub source_updated_at: Option<String>,
    /// Copyright or license information
    pub copyright: Option<String>,
    /// Any additional notes
    pub notes: Option<String>,
}

/// Configuration for a lyrics provider
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// API key or token (if required)
    pub api_key: Option<String>,
    /// Request timeout in seconds
    pub timeout: Duration,
    /// Whether to enable caching
    pub enable_cache: bool,
    /// Maximum number of search results to return
    pub max_results: usize,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            timeout: Duration::from_secs(10),
            enable_cache: true,
            max_results: 10,
        }
    }
}

/// Main trait for lyrics providers
/// 
/// Implement this trait to create a new lyrics source (e.g., Genius, Netease, etc.)
#[async_trait]
pub trait LyricsProvider: Send + Sync {
    /// Returns the unique name of this provider (e.g., "genius", "netease")
    fn name(&self) -> &str;

    /// Returns whether this provider supports synchronized (LRC) lyrics
    fn supports_synced(&self) -> bool {
        false
    }

    /// Returns whether this provider requires authentication
    fn requires_auth(&self) -> bool {
        false
    }

    /// Search for lyrics matching the query
    /// Returns a list of possible matches sorted by relevance
    async fn search(&self, query: &LyricsQuery) -> Result<Vec<LyricsSearchResult>>;

    /// Fetch lyrics by result ID from search results
    async fn fetch(&self, result_id: &str) -> Result<LyricsResponse>;

    /// Convenience method: search and fetch the best match automatically
    /// 
    /// This default implementation:
    /// 1. Searches for matches
    /// 2. Picks the result with highest confidence
    /// 3. Fetches lyrics if confidence is above threshold
    async fn search_and_fetch(&self, query: &LyricsQuery) -> Result<Option<LyricsResponse>> {
        let results = self.search(query).await?;
        
        // Get the highest confidence result
        let best_match = results
            .into_iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap());

        if let Some(result) = best_match {
            // Only fetch if confidence is above threshold (50%)
            if result.confidence > 0.5 {
                tracing::debug!(
                    "Fetching lyrics from {} with confidence {:.2}",
                    self.name(),
                    result.confidence
                );
                return Ok(Some(self.fetch(&result.id).await?));
            } else {
                tracing::debug!(
                    "Skipping fetch from {} - confidence too low: {:.2}",
                    self.name(),
                    result.confidence
                );
            }
        }

        Ok(None)
    }

    /// Health check - verify the provider is accessible
    async fn health_check(&self) -> Result<bool> {
        // Default implementation: try a simple search
        let query = LyricsQuery::new("test");
        self.search(&query).await.map(|_| true)
    }
}

/// Aggregates multiple lyrics providers with fallback logic
pub struct LyricsAggregator {
    providers: Vec<Box<dyn LyricsProvider>>,
}

impl LyricsAggregator {
    /// Create a new empty aggregator
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Add a provider to the aggregator (builder pattern)
    pub fn add_provider(mut self, provider: Box<dyn LyricsProvider>) -> Self {
        self.providers.push(provider);
        self
    }

    /// Add a provider by reference
    pub fn register(&mut self, provider: Box<dyn LyricsProvider>) {
        self.providers.push(provider);
    }

    /// Get list of registered provider names
    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.iter().map(|p| p.name()).collect()
    }

    /// Try to fetch lyrics from all providers in order (with fallback)
    /// 
    /// This tries each provider sequentially until one succeeds.
    /// Useful for reliability when some providers might be down.
    pub async fn fetch_lyrics(&self, query: &LyricsQuery) -> Result<Option<LyricsResponse>> {
        for provider in &self.providers {
            tracing::debug!("Trying provider: {}", provider.name());
            
            match provider.search_and_fetch(query).await {
                Ok(Some(lyrics)) => {
                    tracing::info!("✓ Found lyrics from provider: {}", provider.name());
                    return Ok(Some(lyrics));
                }
                Ok(None) => {
                    tracing::debug!("✗ No lyrics found from provider: {}", provider.name());
                    continue;
                }
                Err(e) => {
                    tracing::warn!(
                        "✗ Provider {} failed: {:?}",
                        provider.name(),
                        e
                    );
                    continue;
                }
            }
        }

        tracing::warn!("No lyrics found from any provider");
        Ok(None)
    }

    /// Try to fetch lyrics from a specific provider by name
    pub async fn fetch_from_provider(
        &self,
        provider_name: &str,
        query: &LyricsQuery,
    ) -> Result<Option<LyricsResponse>> {
        let provider = self
            .providers
            .iter()
            .find(|p| p.name() == provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", provider_name))?;

        provider.search_and_fetch(query).await
    }

    /// Search all providers in parallel and return all results
    /// 
    /// Useful for showing users multiple options to choose from.
    pub async fn search_all(
        &self,
        query: &LyricsQuery,
    ) -> Vec<(String, Result<Vec<LyricsSearchResult>>)> {
        use futures::future::join_all;

        let futures = self.providers.iter().map(|provider| {
            let provider_name = provider.name().to_string();
            let query = query.clone();
            async move {
                let result = provider.search(&query).await;
                (provider_name, result)
            }
        });

        join_all(futures).await
    }

    /// Check health of all providers
    pub async fn health_check_all(&self) -> Vec<(String, bool)> {
        use futures::future::join_all;

        let futures = self.providers.iter().map(|provider| {
            let provider_name = provider.name().to_string();
            async move {
                let healthy = provider.health_check().await.unwrap_or(false);
                (provider_name, healthy)
            }
        });

        join_all(futures).await
    }
}

impl Default for LyricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait to convert between internal types
impl From<LyricsResponse> for Lyric {
    fn from(response: LyricsResponse) -> Self {
        Lyric {
            track_id: String::new(), // Will be set by caller
            content: response.content,
            format: response.format,
            language: response.language,
            source: Some(response.source),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider {
        name: String,
        should_succeed: bool,
        supports_lrc: bool,
    }

    #[async_trait]
    impl LyricsProvider for MockProvider {
        fn name(&self) -> &str {
            &self.name
        }

        fn supports_synced(&self) -> bool {
            self.supports_lrc
        }

        async fn search(&self, query: &LyricsQuery) -> Result<Vec<LyricsSearchResult>> {
            if self.should_succeed {
                Ok(vec![LyricsSearchResult {
                    id: format!("{}-test-id", self.name),
                    title: query.title.clone(),
                    artist: query.artist.clone().unwrap_or_default(),
                    album: query.album.clone(),
                    duration: query.duration,
                    confidence: 0.9,
                }])
            } else {
                anyhow::bail!("Mock failure")
            }
        }

        async fn fetch(&self, _result_id: &str) -> Result<LyricsResponse> {
            Ok(LyricsResponse {
                content: format!("Test lyrics from {}", self.name),
                format: if self.supports_lrc {
                    LyricFormat::Lrc
                } else {
                    LyricFormat::Plain
                },
                language: Some("en".to_string()),
                source: self.name.clone(),
                url: Some(format!("https://{}.example.com", self.name)),
                metadata: LyricsMetadata::default(),
            })
        }
    }

    #[tokio::test]
    async fn test_aggregator_fallback() {
        let aggregator = LyricsAggregator::new()
            .add_provider(Box::new(MockProvider {
                name: "failing".to_string(),
                should_succeed: false,
                supports_lrc: false,
            }))
            .add_provider(Box::new(MockProvider {
                name: "working".to_string(),
                should_succeed: true,
                supports_lrc: true,
            }));

        let query = LyricsQuery::new("Test Song").with_artist("Test Artist");
        let result = aggregator.fetch_lyrics(&query).await.unwrap();

        assert!(result.is_some());
        let lyrics = result.unwrap();
        assert_eq!(lyrics.source, "working");
        assert_eq!(lyrics.format, LyricFormat::Lrc);
    }

    #[tokio::test]
    async fn test_query_builder() {
        let query = LyricsQuery::new("Song Title")
            .with_artist("Artist Name")
            .with_album("Album Name")
            .with_duration(Duration::from_secs(180));

        assert_eq!(query.title, "Song Title");
        assert_eq!(query.artist.unwrap(), "Artist Name");
        assert_eq!(query.album.unwrap(), "Album Name");
        assert_eq!(query.duration.unwrap().as_secs(), 180);
    }

    #[tokio::test]
    async fn test_provider_names() {
        let aggregator = LyricsAggregator::new()
            .add_provider(Box::new(MockProvider {
                name: "provider1".to_string(),
                should_succeed: true,
                supports_lrc: false,
            }))
            .add_provider(Box::new(MockProvider {
                name: "provider2".to_string(),
                should_succeed: true,
                supports_lrc: true,
            }));

        let names = aggregator.provider_names();
        assert_eq!(names, vec!["provider1", "provider2"]);
    }
}
