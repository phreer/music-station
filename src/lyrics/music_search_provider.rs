use super::LyricFormat;
use super::fetcher::{
    LyricsMetadata, LyricsProvider, LyricsQuery, LyricsResponse, LyricsSearchResult,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use music_search_rs::{MusicApi, NetEaseMusicApi, QQMusicApi, SearchType};
use std::time::Duration;

/// Provider for NetEase Cloud Music (网易云音乐)
pub struct NetEaseLyricsProvider {
    api: NetEaseMusicApi,
}

impl NetEaseLyricsProvider {
    pub fn new(cookie: Option<String>) -> Result<Self> {
        let api =
            NetEaseMusicApi::new(cookie).context("Failed to create NetEase Music API client")?;
        Ok(Self { api })
    }
}

#[async_trait]
impl LyricsProvider for NetEaseLyricsProvider {
    fn name(&self) -> &str {
        "netease"
    }

    fn supports_synced(&self) -> bool {
        true // NetEase supports LRC format
    }

    fn requires_auth(&self) -> bool {
        false // Can work without authentication, but some features require it
    }

    async fn search(&self, query: &LyricsQuery) -> Result<Vec<LyricsSearchResult>> {
        // Build search query from title and artist
        let search_query = if let Some(artist) = &query.artist {
            format!("{} {}", query.title, artist)
        } else {
            query.title.clone()
        };

        tracing::debug!("NetEase search query: {}", search_query);

        let result = self.api.search(&search_query, SearchType::SongId).await?;

        if !result.success {
            let error_msg = result
                .error_msg
                .unwrap_or_else(|| "Search failed".to_string());
            anyhow::bail!("NetEase search failed: {}", error_msg);
        }

        let search_data = result.data.context("No search data returned")?;

        let results: Vec<LyricsSearchResult> = search_data
            .song_vos
            .into_iter()
            .map(|song| {
                // Calculate confidence based on title and artist match
                let mut confidence: f32 = 0.5; // Base confidence

                // Increase confidence for title match
                if song
                    .title
                    .to_lowercase()
                    .contains(&query.title.to_lowercase())
                {
                    confidence += 0.3;
                }

                // Increase confidence for artist match
                if let Some(query_artist) = &query.artist {
                    for artist in &song.author_name {
                        if artist.to_lowercase().contains(&query_artist.to_lowercase()) {
                            confidence += 0.2;
                            break;
                        }
                    }
                }

                LyricsSearchResult {
                    id: song.display_id,
                    title: song.title,
                    artist: song.author_name.join(", "),
                    album: Some(song.album_name),
                    duration: Some(Duration::from_millis(song.duration as u64)),
                    confidence: confidence.min(1.0_f32),
                }
            })
            .collect();

        tracing::debug!("NetEase found {} results", results.len());
        Ok(results)
    }

    async fn fetch(&self, result_id: &str) -> Result<LyricsResponse> {
        tracing::debug!("Fetching NetEase lyrics for ID: {}", result_id);

        // Use the MusicApi trait method which returns ResultVo<LyricVo>
        let result: music_search_rs::ResultVo<music_search_rs::LyricVo> =
            <NetEaseMusicApi as MusicApi>::get_lyric(&self.api, "", result_id, false).await?;

        if !result.success {
            let error_msg = result
                .error_msg
                .unwrap_or_else(|| "Lyrics fetch failed".to_string());
            anyhow::bail!("Failed to fetch NetEase lyrics: {}", error_msg);
        }

        let lyric_data = result.data.context("No lyrics data returned")?;

        // Prefer original lyrics, fall back to translated or transliteration
        let content = lyric_data
            .lyric
            .or(lyric_data.translate_lyric.clone())
            .or(lyric_data.transliteration_lyric.clone())
            .context("No lyrics content available")?;

        // Detect format from content (will detect plain, lrc, or lrc_word)
        let format = LyricFormat::detect_from_content(&content);

        // Determine language based on available translations
        let language = if lyric_data.translate_lyric.is_some() {
            Some("zh".to_string()) // Has translation, likely Chinese
        } else {
            None
        };

        Ok(LyricsResponse {
            content,
            format,
            language,
            source: "netease".to_string(),
            url: Some(format!("https://music.163.com/#/song?id={}", result_id)),
            metadata: LyricsMetadata {
                contributor: None,
                source_updated_at: None,
                copyright: Some("NetEase Cloud Music".to_string()),
                notes: if lyric_data.translate_lyric.is_some() {
                    Some("Has translated lyrics available".to_string())
                } else {
                    None
                },
            },
        })
    }
}

/// Provider for QQ Music (QQ音乐)
pub struct QQMusicLyricsProvider {
    api: QQMusicApi,
}

impl QQMusicLyricsProvider {
    pub fn new(cookie: Option<String>) -> Result<Self> {
        let api = QQMusicApi::new(cookie).context("Failed to create QQ Music API client")?;
        Ok(Self { api })
    }
}

#[async_trait]
impl LyricsProvider for QQMusicLyricsProvider {
    fn name(&self) -> &str {
        "qqmusic"
    }

    fn supports_synced(&self) -> bool {
        true // QQ Music supports LRC format
    }

    fn requires_auth(&self) -> bool {
        false
    }

    async fn search(&self, query: &LyricsQuery) -> Result<Vec<LyricsSearchResult>> {
        // Build search query from title and artist
        let search_query = if let Some(artist) = &query.artist {
            format!("{} {}", query.title, artist)
        } else {
            query.title.clone()
        };

        tracing::debug!("QQMusic search query: {}", search_query);

        let result = self.api.search(&search_query, SearchType::SongId).await?;

        if !result.success {
            let error_msg = result
                .error_msg
                .unwrap_or_else(|| "Search failed".to_string());
            anyhow::bail!("QQMusic search failed: {}", error_msg);
        }

        let search_data = result.data.context("No search data returned")?;

        let results: Vec<LyricsSearchResult> = search_data
            .song_vos
            .into_iter()
            .map(|song| {
                // Calculate confidence based on title and artist match
                let mut confidence: f32 = 0.5; // Base confidence

                // Increase confidence for title match
                if song
                    .title
                    .to_lowercase()
                    .contains(&query.title.to_lowercase())
                {
                    confidence += 0.3;
                }

                // Increase confidence for artist match
                if let Some(query_artist) = &query.artist {
                    for artist in &song.author_name {
                        if artist.to_lowercase().contains(&query_artist.to_lowercase()) {
                            confidence += 0.2;
                            break;
                        }
                    }
                }

                LyricsSearchResult {
                    id: song.display_id,
                    title: song.title,
                    artist: song.author_name.join(", "),
                    album: Some(song.album_name),
                    duration: Some(Duration::from_millis(song.duration as u64)),
                    confidence: confidence.min(1.0_f32),
                }
            })
            .collect();

        tracing::debug!("QQMusic found {} results", results.len());
        Ok(results)
    }

    async fn fetch(&self, result_id: &str) -> Result<LyricsResponse> {
        tracing::debug!("Fetching QQMusic lyrics for ID: {}", result_id);

        // Use the MusicApi trait method which returns ResultVo<LyricVo>
        let result: music_search_rs::ResultVo<music_search_rs::LyricVo> =
            <QQMusicApi as MusicApi>::get_lyric(&self.api, result_id, "", false).await?;

        if !result.success {
            let error_msg = result
                .error_msg
                .unwrap_or_else(|| "Lyrics fetch failed".to_string());
            anyhow::bail!("Failed to fetch QQMusic lyrics: {}", error_msg);
        }

        let lyric_data = result.data.context("No lyrics data returned")?;

        // Prefer original lyrics, fall back to translated or transliteration
        let content = lyric_data
            .lyric
            .or(lyric_data.translate_lyric.clone())
            .or(lyric_data.transliteration_lyric.clone())
            .context("No lyrics content available")?;

        // Detect format from content (will detect plain, lrc, or lrc_word)
        let format = LyricFormat::detect_from_content(&content);

        // Determine language based on available translations
        let language = if lyric_data.translate_lyric.is_some() {
            Some("zh".to_string()) // Has translation, likely Chinese
        } else {
            None
        };

        Ok(LyricsResponse {
            content,
            format,
            language,
            source: "qqmusic".to_string(),
            url: Some(format!("https://y.qq.com/n/ryqq/songDetail/{}", result_id)),
            metadata: LyricsMetadata {
                contributor: None,
                source_updated_at: None,
                copyright: Some("QQ Music".to_string()),
                notes: if lyric_data.translate_lyric.is_some() {
                    Some("Has translated lyrics available".to_string())
                } else {
                    None
                },
            },
        })
    }
}
