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

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS play_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                track_id TEXT NOT NULL,
                played_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await
        .context("Failed to create play_events table")?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_play_events_played_at ON play_events(played_at)",
        )
        .execute(&pool)
        .await
        .context("Failed to create play_events index")?;

        tracing::info!("Stats database initialized: {}", db_path.display());

        Ok(Self { pool })
    }

    /// Increment play count for a track
    pub async fn increment_play_count(&self, track_id: &str) -> Result<u64> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .context("Failed to start play transaction")?;
        let row = sqlx::query(
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
        .fetch_one(&mut *transaction)
        .await
        .context("Failed to increment play count")?;
        sqlx::query("INSERT INTO play_events (track_id) VALUES (?)")
            .bind(track_id)
            .execute(&mut *transaction)
            .await
            .context("Failed to record play event")?;
        transaction
            .commit()
            .await
            .context("Failed to commit play event")?;

        let count: i64 = sqlx::Row::get(&row, 0);
        Ok(count as u64)
    }

    /// Return the most played tracks for a calendar month (`YYYY-MM`, UTC).
    pub async fn get_monthly_top_tracks(
        &self,
        month: &str,
        limit: i64,
    ) -> Result<Vec<MonthlyTrackPlays>> {
        let rows = sqlx::query(
            r#"
            SELECT track_id, COUNT(*) AS play_count
            FROM play_events
            WHERE strftime('%Y-%m', played_at) = ?
            GROUP BY track_id
            ORDER BY play_count DESC, track_id ASC
            LIMIT ?
            "#,
        )
        .bind(month)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get monthly top tracks")?;

        Ok(rows
            .into_iter()
            .map(|row| MonthlyTrackPlays {
                track_id: sqlx::Row::get(&row, 0),
                play_count: sqlx::Row::get::<i64, _>(&row, 1) as u64,
            })
            .collect())
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct MonthlyTrackPlays {
    pub track_id: String,
    pub play_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn make_db() -> (StatsDatabase, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db = StatsDatabase::new(&dir.path().join("stats.db"))
            .await
            .unwrap();
        (db, dir)
    }

    #[tokio::test]
    async fn initial_play_count_is_zero() {
        let (db, _dir) = make_db().await;
        assert_eq!(db.get_play_count("nonexistent").await.unwrap(), 0);
    }

    #[tokio::test]
    async fn increment_from_zero() {
        let (db, _dir) = make_db().await;
        let count = db.increment_play_count("track1").await.unwrap();
        assert_eq!(count, 1);
        assert_eq!(db.get_play_count("track1").await.unwrap(), 1);
    }

    #[tokio::test]
    async fn increment_multiple_times() {
        let (db, _dir) = make_db().await;

        for expected in 1..=5 {
            let count = db.increment_play_count("track1").await.unwrap();
            assert_eq!(count, expected);
        }
        assert_eq!(db.get_play_count("track1").await.unwrap(), 5);
    }

    #[tokio::test]
    async fn multiple_tracks_independent() {
        let (db, _dir) = make_db().await;

        db.increment_play_count("a").await.unwrap();
        db.increment_play_count("a").await.unwrap();
        db.increment_play_count("b").await.unwrap();

        assert_eq!(db.get_play_count("a").await.unwrap(), 2);
        assert_eq!(db.get_play_count("b").await.unwrap(), 1);
    }

    #[tokio::test]
    async fn get_all_play_counts() {
        let (db, _dir) = make_db().await;

        // Initially empty
        let counts = db.get_all_play_counts().await.unwrap();
        assert!(counts.is_empty());

        db.increment_play_count("x").await.unwrap();
        db.increment_play_count("x").await.unwrap();
        db.increment_play_count("y").await.unwrap();

        let counts = db.get_all_play_counts().await.unwrap();
        assert_eq!(counts.len(), 2);
        assert_eq!(*counts.get("x").unwrap(), 2);
        assert_eq!(*counts.get("y").unwrap(), 1);
    }

    #[tokio::test]
    async fn monthly_top_tracks_only_counts_events_in_requested_month() {
        let (db, _dir) = make_db().await;

        for (track_id, played_at) in [
            ("popular", "2026-10-01 10:00:00"),
            ("popular", "2026-10-15 10:00:00"),
            ("other", "2026-10-20 10:00:00"),
            ("popular", "2026-09-30 23:59:59"),
        ] {
            sqlx::query("INSERT INTO play_events (track_id, played_at) VALUES (?, ?)")
                .bind(track_id)
                .bind(played_at)
                .execute(&db.pool)
                .await
                .unwrap();
        }

        let tracks = db.get_monthly_top_tracks("2026-10", 10).await.unwrap();
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0].track_id, "popular");
        assert_eq!(tracks[0].play_count, 2);
        assert_eq!(tracks[1].track_id, "other");
        assert_eq!(tracks[1].play_count, 1);
    }
}
