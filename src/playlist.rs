use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tracks: Vec<String>, // Track IDs
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistCreate {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tracks: Option<Vec<String>>,
}

#[derive(Clone)]
pub struct PlaylistDatabase {
    pool: SqlitePool,
}

impl PlaylistDatabase {
    /// Create a new playlist database connection
    pub async fn new(db_path: &Path) -> Result<Self> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            tracing::debug!("Created/verified directory: {}", parent.display());
        }

        // Create database connection with create_if_missing option
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        tracing::debug!("Connecting to playlist database: {}", db_url);

        let pool = SqlitePool::connect(&db_url).await.with_context(|| {
            format!(
                "Failed to connect to playlist database at: {}",
                db_path.display()
            )
        })?;

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .context("Failed to create playlists table")?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS playlist_tracks (
                playlist_id TEXT NOT NULL,
                track_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                PRIMARY KEY (playlist_id, track_id),
                FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await
        .context("Failed to create playlist_tracks table")?;

        // Create index for faster lookups
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_playlist_tracks_playlist_id 
            ON playlist_tracks(playlist_id)
            "#,
        )
        .execute(&pool)
        .await
        .context("Failed to create index")?;

        tracing::info!("Playlist database initialized: {}", db_path.display());

        Ok(Self { pool })
    }

    /// Create a new playlist
    pub async fn create_playlist(&self, create: PlaylistCreate) -> Result<Playlist> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            INSERT INTO playlists (id, name, description, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&create.name)
        .bind(&create.description)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(Playlist {
                id,
                name: create.name,
                description: create.description,
                tracks: Vec::new(),
                created_at: now.clone(),
                updated_at: now,
            }),
            Err(e) => {
                if e.to_string().contains("UNIQUE constraint failed") {
                    anyhow::bail!("A playlist with the name '{}' already exists", create.name)
                } else {
                    Err(e).context("Failed to insert playlist")
                }
            }
        }
    }

    /// Get all playlists
    pub async fn get_playlists(&self) -> Result<Vec<Playlist>> {
        let playlists = sqlx::query_as::<_, (String, String, Option<String>, String, String)>(
            r#"
            SELECT id, name, description, created_at, updated_at
            FROM playlists
            ORDER BY updated_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch playlists")?;

        let mut result = Vec::new();
        for (id, name, description, created_at, updated_at) in playlists {
            let tracks = self.get_playlist_tracks(&id).await?;
            result.push(Playlist {
                id,
                name,
                description,
                tracks,
                created_at,
                updated_at,
            });
        }

        Ok(result)
    }

    /// Get a specific playlist by ID
    pub async fn get_playlist(&self, id: &str) -> Result<Option<Playlist>> {
        let row = sqlx::query_as::<_, (String, String, Option<String>, String, String)>(
            r#"
            SELECT id, name, description, created_at, updated_at
            FROM playlists
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch playlist")?;

        if let Some((id, name, description, created_at, updated_at)) = row {
            let tracks = self.get_playlist_tracks(&id).await?;
            Ok(Some(Playlist {
                id,
                name,
                description,
                tracks,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get track IDs for a playlist
    async fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<String>> {
        let tracks = sqlx::query_as::<_, (String,)>(
            r#"
            SELECT track_id
            FROM playlist_tracks
            WHERE playlist_id = ?
            ORDER BY position
            "#,
        )
        .bind(playlist_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch playlist tracks")?;

        Ok(tracks.into_iter().map(|(track_id,)| track_id).collect())
    }

    /// Update a playlist
    pub async fn update_playlist(
        &self,
        id: &str,
        update: PlaylistUpdate,
    ) -> Result<Option<Playlist>> {
        // Check if playlist exists
        let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM playlists WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .context("Failed to check playlist existence")?;

        if exists == 0 {
            return Ok(None);
        }

        let now = chrono::Utc::now().to_rfc3339();

        // Update playlist metadata if provided
        if update.name.is_some() || update.description.is_some() {
            let current = self
                .get_playlist(id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Playlist not found"))?;

            let name = update.name.unwrap_or(current.name);
            let description = update.description.or(current.description);

            let result = sqlx::query(
                r#"
                UPDATE playlists
                SET name = ?, description = ?, updated_at = ?
                WHERE id = ?
                "#,
            )
            .bind(&name)
            .bind(&description)
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await;

            if let Err(e) = result {
                if e.to_string().contains("UNIQUE constraint failed") {
                    anyhow::bail!("A playlist with the name '{}' already exists", name)
                } else {
                    return Err(e).context("Failed to update playlist");
                }
            }
        }

        // Update tracks if provided
        if let Some(track_ids) = update.tracks {
            // Delete existing tracks
            sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ?")
                .bind(id)
                .execute(&self.pool)
                .await
                .context("Failed to delete old playlist tracks")?;

            // Insert new tracks
            for (position, track_id) in track_ids.iter().enumerate() {
                sqlx::query(
                    r#"
                    INSERT INTO playlist_tracks (playlist_id, track_id, position)
                    VALUES (?, ?, ?)
                    "#,
                )
                .bind(id)
                .bind(track_id)
                .bind(position as i64)
                .execute(&self.pool)
                .await
                .context("Failed to insert playlist track")?;
            }

            // Update the updated_at timestamp
            sqlx::query("UPDATE playlists SET updated_at = ? WHERE id = ?")
                .bind(&now)
                .bind(id)
                .execute(&self.pool)
                .await
                .context("Failed to update playlist timestamp")?;
        }

        // Return updated playlist
        self.get_playlist(id).await
    }

    /// Delete a playlist
    pub async fn delete_playlist(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM playlists WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete playlist")?;

        Ok(result.rows_affected() > 0)
    }

    /// Add a track to a playlist
    pub async fn add_track_to_playlist(
        &self,
        playlist_id: &str,
        track_id: &str,
    ) -> Result<Option<Playlist>> {
        // Check if playlist exists
        let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM playlists WHERE id = ?")
            .bind(playlist_id)
            .fetch_one(&self.pool)
            .await
            .context("Failed to check playlist existence")?;

        if exists == 0 {
            return Ok(None);
        }

        // Get current max position
        let max_position = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT MAX(position) FROM playlist_tracks WHERE playlist_id = ?",
        )
        .bind(playlist_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to get max position")?
        .unwrap_or(-1);

        let new_position = max_position + 1;

        // Insert track (ignore if already exists)
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO playlist_tracks (playlist_id, track_id, position)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(playlist_id)
        .bind(track_id)
        .bind(new_position)
        .execute(&self.pool)
        .await
        .context("Failed to add track to playlist")?;

        // Update timestamp
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE playlists SET updated_at = ? WHERE id = ?")
            .bind(&now)
            .bind(playlist_id)
            .execute(&self.pool)
            .await
            .context("Failed to update playlist timestamp")?;

        self.get_playlist(playlist_id).await
    }

    /// Remove a track from a playlist
    pub async fn remove_track_from_playlist(
        &self,
        playlist_id: &str,
        track_id: &str,
    ) -> Result<Option<Playlist>> {
        let result =
            sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ? AND track_id = ?")
                .bind(playlist_id)
                .bind(track_id)
                .execute(&self.pool)
                .await
                .context("Failed to remove track from playlist")?;

        if result.rows_affected() > 0 {
            // Reorder positions
            let tracks = self.get_playlist_tracks(playlist_id).await?;
            for (position, track_id) in tracks.iter().enumerate() {
                sqlx::query(
                    "UPDATE playlist_tracks SET position = ? WHERE playlist_id = ? AND track_id = ?"
                )
                .bind(position as i64)
                .bind(playlist_id)
                .bind(track_id)
                .execute(&self.pool)
                .await
                .context("Failed to update track position")?;
            }

            // Update timestamp
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query("UPDATE playlists SET updated_at = ? WHERE id = ?")
                .bind(&now)
                .bind(playlist_id)
                .execute(&self.pool)
                .await
                .context("Failed to update playlist timestamp")?;

            self.get_playlist(playlist_id).await
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn make_db() -> (PlaylistDatabase, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db = PlaylistDatabase::new(&dir.path().join("playlists.db"))
            .await
            .unwrap();
        (db, dir)
    }

    #[tokio::test]
    async fn crud_create_get_delete() {
        let (db, _dir) = make_db().await;

        // Initially empty
        let all = db.get_playlists().await.unwrap();
        assert!(all.is_empty());

        // Create
        let pl = db
            .create_playlist(PlaylistCreate {
                name: "My Playlist".into(),
                description: Some("Test description".into()),
            })
            .await
            .unwrap();
        assert_eq!(pl.name, "My Playlist");
        assert_eq!(pl.description.as_deref(), Some("Test description"));
        assert!(pl.tracks.is_empty());

        // Get by ID
        let fetched = db.get_playlist(&pl.id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "My Playlist");

        // List all
        let all = db.get_playlists().await.unwrap();
        assert_eq!(all.len(), 1);

        // Delete
        assert!(db.delete_playlist(&pl.id).await.unwrap());
        assert!(db.get_playlist(&pl.id).await.unwrap().is_none());

        // Delete non-existent
        assert!(!db.delete_playlist(&pl.id).await.unwrap());
    }

    #[tokio::test]
    async fn duplicate_name_rejected() {
        let (db, _dir) = make_db().await;

        db.create_playlist(PlaylistCreate {
            name: "Dups".into(),
            description: None,
        })
        .await
        .unwrap();

        let err = db
            .create_playlist(PlaylistCreate {
                name: "Dups".into(),
                description: None,
            })
            .await;
        assert!(err.is_err());
        assert!(err
            .unwrap_err()
            .to_string()
            .contains("already exists"));
    }

    #[tokio::test]
    async fn update_playlist_metadata() {
        let (db, _dir) = make_db().await;

        let pl = db
            .create_playlist(PlaylistCreate {
                name: "Original".into(),
                description: None,
            })
            .await
            .unwrap();

        let updated = db
            .update_playlist(
                &pl.id,
                PlaylistUpdate {
                    name: Some("Renamed".into()),
                    description: Some("New desc".into()),
                    tracks: None,
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.name, "Renamed");
        assert_eq!(updated.description.as_deref(), Some("New desc"));
    }

    #[tokio::test]
    async fn update_nonexistent_returns_none() {
        let (db, _dir) = make_db().await;

        let result = db
            .update_playlist(
                "no-such-id",
                PlaylistUpdate {
                    name: Some("X".into()),
                    description: None,
                    tracks: None,
                },
            )
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn add_and_remove_tracks() {
        let (db, _dir) = make_db().await;

        let pl = db
            .create_playlist(PlaylistCreate {
                name: "Tracks Test".into(),
                description: None,
            })
            .await
            .unwrap();

        // Add tracks
        db.add_track_to_playlist(&pl.id, "t1").await.unwrap();
        db.add_track_to_playlist(&pl.id, "t2").await.unwrap();
        db.add_track_to_playlist(&pl.id, "t3").await.unwrap();

        let fetched = db.get_playlist(&pl.id).await.unwrap().unwrap();
        assert_eq!(fetched.tracks, vec!["t1", "t2", "t3"]);

        // Remove middle track
        db.remove_track_from_playlist(&pl.id, "t2").await.unwrap();

        let fetched = db.get_playlist(&pl.id).await.unwrap().unwrap();
        assert_eq!(fetched.tracks, vec!["t1", "t3"]);
    }

    #[tokio::test]
    async fn add_duplicate_track_ignored() {
        let (db, _dir) = make_db().await;

        let pl = db
            .create_playlist(PlaylistCreate {
                name: "Dup Track".into(),
                description: None,
            })
            .await
            .unwrap();

        db.add_track_to_playlist(&pl.id, "t1").await.unwrap();
        db.add_track_to_playlist(&pl.id, "t1").await.unwrap(); // duplicate

        let fetched = db.get_playlist(&pl.id).await.unwrap().unwrap();
        assert_eq!(fetched.tracks, vec!["t1"]);
    }

    #[tokio::test]
    async fn add_track_to_nonexistent_playlist() {
        let (db, _dir) = make_db().await;

        let result = db.add_track_to_playlist("no-such-id", "t1").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn remove_track_from_nonexistent_returns_none() {
        let (db, _dir) = make_db().await;

        let pl = db
            .create_playlist(PlaylistCreate {
                name: "P".into(),
                description: None,
            })
            .await
            .unwrap();

        // Remove a track that was never added
        let result = db
            .remove_track_from_playlist(&pl.id, "no-such-track")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn update_playlist_tracks_via_update() {
        let (db, _dir) = make_db().await;

        let pl = db
            .create_playlist(PlaylistCreate {
                name: "Bulk Update".into(),
                description: None,
            })
            .await
            .unwrap();

        db.add_track_to_playlist(&pl.id, "old1").await.unwrap();

        // Replace tracks entirely via update
        let updated = db
            .update_playlist(
                &pl.id,
                PlaylistUpdate {
                    name: None,
                    description: None,
                    tracks: Some(vec!["new1".into(), "new2".into()]),
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.tracks, vec!["new1", "new2"]);
    }

    #[tokio::test]
    async fn delete_playlist_cascades_tracks() {
        let (db, _dir) = make_db().await;

        let pl = db
            .create_playlist(PlaylistCreate {
                name: "Cascade".into(),
                description: None,
            })
            .await
            .unwrap();

        db.add_track_to_playlist(&pl.id, "t1").await.unwrap();
        db.add_track_to_playlist(&pl.id, "t2").await.unwrap();

        // Delete playlist — tracks should be cascade-deleted
        assert!(db.delete_playlist(&pl.id).await.unwrap());
        assert!(db.get_playlist(&pl.id).await.unwrap().is_none());
    }
}
