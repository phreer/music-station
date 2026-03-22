use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteArtist {
    pub artist_name: String,
    pub favorited_at: String,
}

#[derive(Clone)]
pub struct FavoritesDatabase {
    pool: SqlitePool,
}

impl FavoritesDatabase {
    /// Create a new favorites database connection
    pub async fn new(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await.with_context(|| {
            format!(
                "Failed to connect to favorites database at: {}",
                db_path.display()
            )
        })?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS favorite_artists (
                artist_name TEXT PRIMARY KEY,
                favorited_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await
        .context("Failed to create favorite_artists table")?;

        tracing::info!("Favorites database initialized: {}", db_path.display());

        Ok(Self { pool })
    }

    /// Add an artist to favorites. Idempotent — does nothing if already favorited.
    pub async fn add_favorite_artist(&self, artist_name: &str) -> Result<FavoriteArtist> {
        sqlx::query(
            r#"
            INSERT INTO favorite_artists (artist_name, favorited_at)
            VALUES (?, datetime('now'))
            ON CONFLICT(artist_name) DO NOTHING
            "#,
        )
        .bind(artist_name)
        .execute(&self.pool)
        .await
        .context("Failed to add favorite artist")?;

        self.get_favorite_artist(artist_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve newly added favorite artist"))
    }

    /// Remove an artist from favorites. Returns true if it was actually removed.
    pub async fn remove_favorite_artist(&self, artist_name: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM favorite_artists WHERE artist_name = ?")
            .bind(artist_name)
            .execute(&self.pool)
            .await
            .context("Failed to remove favorite artist")?;

        Ok(result.rows_affected() > 0)
    }

    /// Check whether an artist is favorited.
    pub async fn is_favorite_artist(&self, artist_name: &str) -> Result<bool> {
        let row = sqlx::query(
            "SELECT 1 FROM favorite_artists WHERE artist_name = ? LIMIT 1",
        )
        .bind(artist_name)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to check favorite artist")?;

        Ok(row.is_some())
    }

    /// Get all favorite artists ordered by most recently favorited first.
    pub async fn get_favorite_artists(&self) -> Result<Vec<FavoriteArtist>> {
        let rows = sqlx::query(
            "SELECT artist_name, favorited_at FROM favorite_artists ORDER BY favorited_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get favorite artists")?;

        Ok(rows
            .into_iter()
            .map(|row| FavoriteArtist {
                artist_name: sqlx::Row::get(&row, 0),
                favorited_at: sqlx::Row::get(&row, 1),
            })
            .collect())
    }

    /// Get all favorited artist names as a set, for efficient bulk lookup.
    pub async fn get_favorite_artist_names(&self) -> Result<HashSet<String>> {
        let rows = sqlx::query("SELECT artist_name FROM favorite_artists")
            .fetch_all(&self.pool)
            .await
            .context("Failed to get favorite artist names")?;

        Ok(rows
            .into_iter()
            .map(|row| sqlx::Row::get(&row, 0))
            .collect())
    }

    async fn get_favorite_artist(&self, artist_name: &str) -> Result<Option<FavoriteArtist>> {
        let row = sqlx::query(
            "SELECT artist_name, favorited_at FROM favorite_artists WHERE artist_name = ?",
        )
        .bind(artist_name)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get favorite artist")?;

        Ok(row.map(|r| FavoriteArtist {
            artist_name: sqlx::Row::get(&r, 0),
            favorited_at: sqlx::Row::get(&r, 1),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn make_db() -> (FavoritesDatabase, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db = FavoritesDatabase::new(&dir.path().join("favorites.db"))
            .await
            .unwrap();
        (db, dir)
    }

    #[tokio::test]
    async fn initially_no_favorites() {
        let (db, _dir) = make_db().await;
        assert!(db.get_favorite_artists().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn add_and_check_favorite() {
        let (db, _dir) = make_db().await;
        db.add_favorite_artist("Pink Floyd").await.unwrap();
        assert!(db.is_favorite_artist("Pink Floyd").await.unwrap());
        assert!(!db.is_favorite_artist("Beatles").await.unwrap());
    }

    #[tokio::test]
    async fn add_is_idempotent() {
        let (db, _dir) = make_db().await;
        db.add_favorite_artist("Pink Floyd").await.unwrap();
        db.add_favorite_artist("Pink Floyd").await.unwrap();
        let all = db.get_favorite_artists().await.unwrap();
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn remove_favorite() {
        let (db, _dir) = make_db().await;
        db.add_favorite_artist("Pink Floyd").await.unwrap();
        let removed = db.remove_favorite_artist("Pink Floyd").await.unwrap();
        assert!(removed);
        assert!(!db.is_favorite_artist("Pink Floyd").await.unwrap());
    }

    #[tokio::test]
    async fn remove_nonexistent_returns_false() {
        let (db, _dir) = make_db().await;
        let removed = db.remove_favorite_artist("Nobody").await.unwrap();
        assert!(!removed);
    }

    #[tokio::test]
    async fn get_favorite_artist_names() {
        let (db, _dir) = make_db().await;
        db.add_favorite_artist("Artist A").await.unwrap();
        db.add_favorite_artist("Artist B").await.unwrap();
        let names = db.get_favorite_artist_names().await.unwrap();
        assert_eq!(names.len(), 2);
        assert!(names.contains("Artist A"));
        assert!(names.contains("Artist B"));
    }
}
