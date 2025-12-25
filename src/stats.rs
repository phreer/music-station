use anyhow::{Context, Result};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone)]
pub struct StatsDatabase {
    pool: SqlitePool,
}

impl StatsDatabase {
    /// Create a new stats database connection
    pub async fn new(db_path: &Path) -> Result<Self> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Create database connection
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await.with_context(|| {
            format!(
                "Failed to connect to stats database at: {}",
                db_path.display()
            )
        })?;

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS track_stats (
                track_id TEXT PRIMARY KEY,
                play_count INTEGER NOT NULL DEFAULT 0,
                last_played_at TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .context("Failed to create track_stats table")?;

        tracing::info!("Stats database initialized: {}", db_path.display());

        Ok(Self { pool })
    }

    /// Increment play count for a track
    pub async fn increment_play_count(&self, track_id: &str) -> Result<u64> {
        sqlx::query(
            r#"
            INSERT INTO track_stats (track_id, play_count, last_played_at)
            VALUES (?, 1, datetime('now'))
            ON CONFLICT(track_id) DO UPDATE SET
                play_count = play_count + 1,
                last_played_at = datetime('now')
            RETURNING play_count
            "#,
        )
        .bind(track_id)
        .fetch_one(&self.pool)
        .await
        .map(|row| {
            let count: i64 = sqlx::Row::get(&row, 0);
            count as u64
        })
        .context("Failed to increment play count")
    }

    /// Get play count for a track
    pub async fn get_play_count(&self, track_id: &str) -> Result<u64> {
        let row = sqlx::query("SELECT play_count FROM track_stats WHERE track_id = ?")
            .bind(track_id)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to get play count")?;

        Ok(row
            .map(|r| {
                let count: i64 = sqlx::Row::get(&r, 0);
                count as u64
            })
            .unwrap_or(0))
    }

    /// Get all play counts
    pub async fn get_all_play_counts(&self) -> Result<HashMap<String, u64>> {
        let rows = sqlx::query("SELECT track_id, play_count FROM track_stats")
            .fetch_all(&self.pool)
            .await
            .context("Failed to get all play counts")?;

        let mut counts = HashMap::new();
        for row in rows {
            let id: String = sqlx::Row::get(&row, 0);
            let count: i64 = sqlx::Row::get(&row, 1);
            counts.insert(id, count as u64);
        }
        Ok(counts)
    }
}
