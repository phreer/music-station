pub mod fetcher;
pub mod providers;
pub mod music_search_provider;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lyric {
    pub track_id: String,
    pub content: String,
    pub format: LyricFormat,
    pub language: Option<String>,
    pub source: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LyricFormat {
    Plain,
    Lrc,  // Standard LRC (Lyrics) format with line-level timestamps
    #[serde(rename = "lrc_word")]
    LrcWord,  // Extended LRC format with word-level timestamps
}

impl LyricFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            LyricFormat::Plain => "plain",
            LyricFormat::Lrc => "lrc",
            LyricFormat::LrcWord => "lrc_word",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "lrc" => LyricFormat::Lrc,
            "lrc_word" | "lrcword" | "word" | "extended" => LyricFormat::LrcWord,
            _ => LyricFormat::Plain,
        }
    }
    
    /// Detect format from content automatically
    pub fn detect_from_content(content: &str) -> Self {
        // Check for word-level timing pattern: word(offset,duration)
        if content.contains("(") && content.contains(",") && content.contains(")") {
            // Look for pattern like: word(123,456)
            let word_timing_regex = regex::Regex::new(r"\S+\(\d+,\d+\)").unwrap();
            if word_timing_regex.is_match(content) {
                return LyricFormat::LrcWord;
            }
        }
        
        // Check for standard LRC timing pattern: [mm:ss.xx] or [offset,duration]
        if content.contains("[") && (content.contains(":") || content.contains(",")) {
            // Look for patterns like [00:12.34] or [12345,6789]
            let lrc_regex = regex::Regex::new(r"\[\d+:\d{2}\.\d{2,3}\]|\[\d+,\d+\]").unwrap();
            if lrc_regex.is_match(content) {
                // It's LRC format, but check if it has word-level timing
                let word_timing_regex = regex::Regex::new(r"\S+\(\d+,\d+\)").unwrap();
                if word_timing_regex.is_match(content) {
                    return LyricFormat::LrcWord;
                }
                return LyricFormat::Lrc;
            }
        }
        
        // Default to plain text
        LyricFormat::Plain
    }
}

#[derive(Debug, Clone)]
pub struct LyricDatabase {
    pool: SqlitePool,
}

impl LyricDatabase {
    /// Create a new lyric database connection
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_path = db_path.as_ref();
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let connection_string = format!("sqlite:{}", db_path.display());
        let options = SqliteConnectOptions::from_str(&connection_string)?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .context("Failed to connect to lyrics database")?;

        let db = Self { pool };
        db.initialize().await?;

        Ok(db)
    }

    /// Initialize database schema
    async fn initialize(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS lyrics (
                track_id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                format TEXT NOT NULL,
                language TEXT,
                source TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create lyrics table")?;

        Ok(())
    }

    /// Save or update lyrics for a track
    pub async fn save_lyric(
        &self,
        track_id: &str,
        content: String,
        format: LyricFormat,
        language: Option<String>,
        source: Option<String>,
    ) -> Result<Lyric> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO lyrics (track_id, content, format, language, source, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(track_id) DO UPDATE SET
                content = excluded.content,
                format = excluded.format,
                language = excluded.language,
                source = excluded.source,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(track_id)
        .bind(&content)
        .bind(format.as_str())
        .bind(&language)
        .bind(&source)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .context("Failed to save lyric")?;

        Ok(Lyric {
            track_id: track_id.to_string(),
            content,
            format,
            language,
            source,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Get lyrics for a specific track
    pub async fn get_lyric(&self, track_id: &str) -> Result<Option<Lyric>> {
        let row = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, String, String)>(
            r#"
            SELECT track_id, content, format, language, source, created_at, updated_at
            FROM lyrics
            WHERE track_id = ?
            "#,
        )
        .bind(track_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch lyric")?;

        Ok(row.map(|(track_id, content, format, language, source, created_at, updated_at)| {
            Lyric {
                track_id,
                content,
                format: LyricFormat::from_str(&format),
                language,
                source,
                created_at,
                updated_at,
            }
        }))
    }

    /// Delete lyrics for a track
    pub async fn delete_lyric(&self, track_id: &str) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM lyrics
            WHERE track_id = ?
            "#,
        )
        .bind(track_id)
        .execute(&self.pool)
        .await
        .context("Failed to delete lyric")?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if a track has lyrics
    pub async fn has_lyric(&self, track_id: &str) -> Result<bool> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM lyrics WHERE track_id = ?
            "#,
        )
        .bind(track_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to check lyric existence")?;

        Ok(count.0 > 0)
    }

    /// Get all track IDs that have lyrics
    pub async fn get_tracks_with_lyrics(&self) -> Result<Vec<String>> {
        let rows = sqlx::query_as::<_, (String,)>(
            r#"
            SELECT track_id FROM lyrics ORDER BY updated_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch tracks with lyrics")?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    /// Get statistics about lyrics in the database
    pub async fn get_stats(&self) -> Result<LyricStats> {
        let (total_lyrics,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM lyrics"
        )
        .fetch_one(&self.pool)
        .await?;

        let (lrc_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM lyrics WHERE format = 'lrc'"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(LyricStats {
            total_lyrics: total_lyrics as usize,
            lrc_format_count: lrc_count as usize,
            plain_format_count: (total_lyrics - lrc_count) as usize,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LyricStats {
    pub total_lyrics: usize,
    pub lrc_format_count: usize,
    pub plain_format_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct LyricUpload {
    pub content: String,
    #[serde(default)]
    pub format: Option<String>,
    pub language: Option<String>,
    pub source: Option<String>,
}
