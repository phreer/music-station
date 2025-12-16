use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::audio::get_audio_file_handler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub path: PathBuf,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub year: Option<String>,
    pub track_number: Option<String>,
    pub disc_number: Option<String>,
    pub composer: Option<String>,
    pub comment: Option<String>,
    pub duration_secs: Option<u64>,
    pub file_size: u64,
    pub has_cover: bool,
    pub has_lyrics: bool,
    pub custom_fields: HashMap<String, String>,
}

// Re-export the MetadataUpdate from audio module for API compatibility
pub use crate::audio::MetadataUpdate as TrackMetadataUpdate;

#[derive(Debug, Clone, Serialize)]
pub struct Album {
    pub name: String,
    pub artist: Option<String>,
    pub track_count: usize,
    pub total_duration_secs: u64,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Artist {
    pub name: String,
    pub album_count: usize,
    pub track_count: usize,
    pub albums: Vec<Album>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LibraryStats {
    pub total_tracks: usize,
    pub total_albums: usize,
    pub total_artists: usize,
    pub total_duration_secs: u64,
    pub total_size_bytes: u64,
}

#[derive(Clone)]
pub struct MusicLibrary {
    library_path: PathBuf,
    tracks: Arc<RwLock<Vec<Track>>>,
}

impl MusicLibrary {
    pub fn new(library_path: PathBuf) -> Self {
        Self {
            library_path,
            tracks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Scan the library folder for audio files (FLAC and MP3)
    pub async fn scan(&self) -> Result<()> {
        tracing::info!("Scanning library at: {}", self.library_path.display());

        let mut tracks = Vec::new();
        Box::pin(self.scan_directory(&self.library_path.clone(), &mut tracks)).await?;

        let mut library_tracks = self.tracks.write().await;
        *library_tracks = tracks;

        tracing::info!("Scan complete. Found {} tracks", library_tracks.len());
        Ok(())
    }

    /// Recursively scan a directory for audio files
    fn scan_directory<'a>(
        &'a self,
        dir: &'a Path,
        tracks: &'a mut Vec<Track>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            let mut entries = tokio::fs::read_dir(dir)
                .await
                .context(format!("Failed to read directory: {}", dir.display()))?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let metadata = tokio::fs::metadata(&path).await?;

                if metadata.is_dir() {
                    // Recursively scan subdirectories
                    tracing::debug!("Scanning subdirectory: {}", path.display());
                    self.scan_directory(&path, tracks).await?;
                } else if metadata.is_file() {
                    // Process audio files
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        if ext == "flac" || ext == "mp3" || ext == "ogg" {
                            match self.parse_audio_file(&path).await {
                                Ok(track) => {
                                    tracing::info!(
                                        "Found track: {} - {}",
                                        track.artist.as_deref().unwrap_or("Unknown Artist"),
                                        track.title.as_deref().unwrap_or("Unknown")
                                    );
                                    tracks.push(track);
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to parse {}: {}", path.display(), e);
                                }
                            }
                        }
                    }
                }
            }

            Ok(())
        })
    }

    /// Parse an audio file (FLAC or MP3) and extract metadata
    async fn parse_audio_file(&self, path: &Path) -> Result<Track> {
        let metadata = tokio::fs::metadata(path).await?;
        let file_size = metadata.len();

        // Get the appropriate audio file handler based on extension
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("No file extension"))?;

        let handler = get_audio_file_handler(ext)
            .ok_or_else(|| anyhow::anyhow!("Unsupported file format: {}", ext))?;

        // Parse metadata using the handler
        let audio_metadata = handler
            .parse_metadata(path)
            .context("Failed to parse audio metadata")?;

        // Check for embedded cover art
        let has_cover = handler.has_cover_art(path).unwrap_or(false);

        // Generate a unique ID from the relative path (relative to library directory)
        // This ensures consistent IDs regardless of where the library is mounted
        let relative_path = path
            .strip_prefix(&self.library_path)
            .unwrap_or(path)
            .to_string_lossy();
        let id = format!("{:x}", md5::compute(relative_path.as_bytes()));

        Ok(Track {
            id,
            path: path.to_path_buf(),
            title: audio_metadata.title,
            artist: audio_metadata.artist,
            album: audio_metadata.album,
            album_artist: audio_metadata.album_artist,
            genre: audio_metadata.genre,
            year: audio_metadata.year,
            track_number: audio_metadata.track_number,
            disc_number: audio_metadata.disc_number,
            composer: audio_metadata.composer,
            comment: audio_metadata.comment,
            duration_secs: audio_metadata.duration_secs,
            file_size,
            has_cover,
            has_lyrics: false, // Will be updated when lyrics database is queried
            custom_fields: audio_metadata.custom_fields,
        })
    }

    /// Get all tracks in the library
    pub async fn get_tracks(&self) -> Vec<Track> {
        self.tracks.read().await.clone()
    }

    /// Get a specific track by ID
    pub async fn get_track(&self, id: &str) -> Option<Track> {
        self.tracks
            .read()
            .await
            .iter()
            .find(|t| t.id == id)
            .cloned()
    }

    /// Get the library path
    #[allow(dead_code)]
    pub fn library_path(&self) -> &Path {
        &self.library_path
    }

    /// Update the has_lyrics flag for a track
    pub async fn update_track_lyrics_status(&self, track_id: &str, has_lyrics: bool) {
        let mut tracks = self.tracks.write().await;
        if let Some(track) = tracks.iter_mut().find(|t| t.id == track_id) {
            track.has_lyrics = has_lyrics;
        }
    }

    /// Get all albums in the library
    pub async fn get_albums(&self) -> Vec<Album> {
        use std::collections::HashMap;

        let tracks = self.tracks.read().await;
        let mut albums_map: HashMap<String, Vec<Track>> = HashMap::new();

        // Group tracks by album
        for track in tracks.iter() {
            let album_name = track
                .album
                .clone()
                .unwrap_or_else(|| "Unknown Album".to_string());
            albums_map
                .entry(album_name)
                .or_insert_with(Vec::new)
                .push(track.clone());
        }

        // Convert to Album structs
        let mut albums: Vec<Album> = albums_map
            .into_iter()
            .map(|(name, tracks)| {
                let artist = tracks.first().and_then(|t| t.artist.clone());
                let total_duration_secs = tracks.iter().filter_map(|t| t.duration_secs).sum();
                let track_count = tracks.len();

                Album {
                    name,
                    artist,
                    track_count,
                    total_duration_secs,
                    tracks,
                }
            })
            .collect();

        // Sort by album name
        albums.sort_by(|a, b| a.name.cmp(&b.name));
        albums
    }

    /// Get all artists with their albums
    pub async fn get_artists(&self) -> Vec<Artist> {
        use std::collections::HashMap;

        let albums = self.get_albums().await;
        let mut artists_map: HashMap<String, Vec<Album>> = HashMap::new();

        // Group albums by artist
        for album in albums {
            let artist_name = album
                .artist
                .clone()
                .unwrap_or_else(|| "Unknown Artist".to_string());
            artists_map
                .entry(artist_name)
                .or_insert_with(Vec::new)
                .push(album);
        }

        // Convert to Artist structs
        let mut artists: Vec<Artist> = artists_map
            .into_iter()
            .map(|(name, albums)| {
                let album_count = albums.len();
                let track_count = albums.iter().map(|a| a.track_count).sum();

                Artist {
                    name,
                    album_count,
                    track_count,
                    albums,
                }
            })
            .collect();

        // Sort by artist name
        artists.sort_by(|a, b| a.name.cmp(&b.name));
        artists
    }

    /// Get a specific album by name
    pub async fn get_album(&self, album_name: &str) -> Option<Album> {
        self.get_albums()
            .await
            .into_iter()
            .find(|a| a.name == album_name)
    }

    /// Get a specific artist by name
    pub async fn get_artist(&self, artist_name: &str) -> Option<Artist> {
        self.get_artists()
            .await
            .into_iter()
            .find(|a| a.name == artist_name)
    }

    /// Get library statistics
    pub async fn get_stats(&self) -> LibraryStats {
        let tracks = self.tracks.read().await;
        let albums = self.get_albums().await;
        let artists = self.get_artists().await;

        let total_duration_secs = tracks.iter().filter_map(|t| t.duration_secs).sum();
        let total_size_bytes = tracks.iter().map(|t| t.file_size).sum();

        LibraryStats {
            total_tracks: tracks.len(),
            total_albums: albums.len(),
            total_artists: artists.len(),
            total_duration_secs,
            total_size_bytes,
        }
    }

    /// Update metadata for a track
    pub async fn update_track_metadata(
        &self,
        id: &str,
        update: TrackMetadataUpdate,
    ) -> Result<Track> {
        // Find the track
        let track = {
            let tracks = self.tracks.read().await;
            tracks
                .iter()
                .find(|t| t.id == id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Track not found: {}", id))?
        };

        tracing::debug!(
            "Updating metadata for track: {} ({})",
            track.title.as_deref().unwrap_or("Unknown"),
            track.path.display()
        );

        // Update the audio file metadata
        self.write_audio_metadata(&track.path, &update)
            .context(format!(
                "Failed to write metadata to file: {}",
                track.path.display()
            ))?;

        // Re-parse the file to get updated metadata
        let mut updated_track = self
            .parse_audio_file(&track.path)
            .await
            .context("Failed to re-parse file after update")?;

        // Preserve the has_lyrics flag from the original track
        // (it's stored in the lyrics database, not in the audio file)
        updated_track.has_lyrics = track.has_lyrics;

        // Update in-memory track list
        {
            let mut tracks = self.tracks.write().await;
            if let Some(pos) = tracks.iter().position(|t| t.id == id) {
                tracks[pos] = updated_track.clone();
            }
        }

        tracing::info!(
            "Updated metadata for track: {} ({})",
            updated_track.title.as_deref().unwrap_or("Unknown"),
            id
        );

        Ok(updated_track)
    }

    /// Write metadata to an audio file (FLAC or MP3)
    fn write_audio_metadata(&self, path: &Path, update: &TrackMetadataUpdate) -> Result<()> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("No file extension"))?;

        tracing::debug!("Writing metadata to {} file: {}", ext, path.display());

        let handler = get_audio_file_handler(ext)
            .ok_or_else(|| anyhow::anyhow!("Unsupported file format: {}", ext))?;

        handler
            .write_metadata(path, update)
            .context(format!("Failed to write metadata to {}", path.display()))
    }

    /// Get cover art from an audio file (FLAC or MP3)
    pub fn get_cover_art(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("No file extension"))?;

        let handler = get_audio_file_handler(ext)
            .ok_or_else(|| anyhow::anyhow!("Unsupported file format: {}", ext))?;

        handler.get_cover_art(path)
    }

    /// Set cover art for an audio file (FLAC or MP3)
    pub async fn set_cover_art(
        &self,
        id: &str,
        image_data: Vec<u8>,
        mime_type: &str,
    ) -> Result<()> {
        // Find the track
        let track = {
            let tracks = self.tracks.read().await;
            tracks
                .iter()
                .find(|t| t.id == id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Track not found"))?
        };

        let ext = track
            .path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("No file extension"))?;

        let handler = get_audio_file_handler(ext)
            .ok_or_else(|| anyhow::anyhow!("Unsupported file format: {}", ext))?;

        handler.set_cover_art(&track.path, image_data, mime_type)?;

        // Update in-memory track
        let mut updated_track = self
            .parse_audio_file(&track.path)
            .await
            .context("Failed to re-parse file after cover update")?;

        // Preserve the has_lyrics flag
        updated_track.has_lyrics = track.has_lyrics;

        {
            let mut tracks = self.tracks.write().await;
            if let Some(pos) = tracks.iter().position(|t| t.id == id) {
                tracks[pos] = updated_track;
            }
        }

        tracing::info!("Updated cover art for track: {}", id);

        Ok(())
    }

    /// Remove cover art from an audio file (FLAC or MP3)
    pub async fn remove_cover_art(&self, id: &str) -> Result<()> {
        // Find the track
        let track = {
            let tracks = self.tracks.read().await;
            tracks
                .iter()
                .find(|t| t.id == id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Track not found"))?
        };

        let ext = track
            .path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("No file extension"))?;

        let handler = get_audio_file_handler(ext)
            .ok_or_else(|| anyhow::anyhow!("Unsupported file format: {}", ext))?;

        handler.remove_cover_art(&track.path)?;

        // Update in-memory track
        let mut updated_track = self
            .parse_audio_file(&track.path)
            .await
            .context("Failed to re-parse file after cover removal")?;

        // Preserve the has_lyrics flag
        updated_track.has_lyrics = track.has_lyrics;

        {
            let mut tracks = self.tracks.write().await;
            if let Some(pos) = tracks.iter().position(|t| t.id == id) {
                tracks[pos] = updated_track;
            }
        }

        tracing::info!("Removed cover art for track: {}", id);

        Ok(())
    }
}
