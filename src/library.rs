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
    pub play_count: u64,
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
    pub total_plays: u64,
}

#[derive(Clone)]
pub struct MusicLibrary {
    library_path: PathBuf,
    tracks: Arc<RwLock<Vec<Track>>>,
    albums_cache: Arc<RwLock<Option<Vec<Album>>>>,
    artists_cache: Arc<RwLock<Option<Vec<Artist>>>>,
}

impl MusicLibrary {
    pub fn new(library_path: PathBuf) -> Self {
        Self {
            library_path,
            tracks: Arc::new(RwLock::new(Vec::new())),
            albums_cache: Arc::new(RwLock::new(None)),
            artists_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Invalidate the cached album and artist collections.
    /// Must be called whenever the track list is mutated.
    async fn invalidate_cache(&self) {
        *self.albums_cache.write().await = None;
        *self.artists_cache.write().await = None;
    }

    /// Scan the library folder for audio files (FLAC and MP3)
    pub async fn scan(&self) -> Result<()> {
        tracing::info!("Scanning library at: {}", self.library_path.display());

        let mut tracks = Vec::new();
        Box::pin(self.scan_directory(&self.library_path.clone(), &mut tracks)).await?;

        let mut library_tracks = self.tracks.write().await;
        *library_tracks = tracks;
        drop(library_tracks);
        self.invalidate_cache().await;

        let track_count = self.tracks.read().await.len();
        tracing::info!("Scan complete. Found {} tracks", track_count);
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

        // Parse metadata and check cover art in a blocking task
        // (audio libraries use synchronous I/O internally)
        let path_owned = path.to_path_buf();
        let (audio_metadata, has_cover) = tokio::task::spawn_blocking(move || {
            let audio_metadata = handler
                .parse_metadata(&path_owned)
                .context("Failed to parse audio metadata")?;
            let has_cover = handler.has_cover_art(&path_owned).unwrap_or(false);
            Ok::<_, anyhow::Error>((audio_metadata, has_cover))
        })
        .await??;

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
            play_count: 0,     // Will be updated when stats database is queried
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
        drop(tracks);
        self.invalidate_cache().await;
    }

    /// Update the play count for a track
    pub async fn update_track_play_count(&self, track_id: &str, play_count: u64) {
        let mut tracks = self.tracks.write().await;
        if let Some(track) = tracks.iter_mut().find(|t| t.id == track_id) {
            track.play_count = play_count;
        }
        drop(tracks);
        self.invalidate_cache().await;
    }

    /// Build the album list from tracks (uncached computation).
    fn build_albums(tracks: &[Track]) -> Vec<Album> {
        let mut albums_map: HashMap<String, Vec<Track>> = HashMap::new();

        for track in tracks.iter() {
            let album_name = track
                .album
                .clone()
                .unwrap_or_else(|| "Unknown Album".to_string());
            albums_map
                .entry(album_name)
                .or_default()
                .push(track.clone());
        }

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

        albums.sort_by(|a, b| a.name.cmp(&b.name));
        albums
    }

    /// Build the artist list from albums (uncached computation).
    fn build_artists(albums: &[Album]) -> Vec<Artist> {
        let mut artists_map: HashMap<String, Vec<Album>> = HashMap::new();

        for album in albums.iter() {
            let artist_name = album
                .artist
                .clone()
                .unwrap_or_else(|| "Unknown Artist".to_string());
            artists_map
                .entry(artist_name)
                .or_default()
                .push(album.clone());
        }

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

        artists.sort_by(|a, b| a.name.cmp(&b.name));
        artists
    }

    /// Get all albums in the library (cached).
    pub async fn get_albums(&self) -> Vec<Album> {
        {
            let cache = self.albums_cache.read().await;
            if let Some(ref albums) = *cache {
                return albums.clone();
            }
        }

        let tracks = self.tracks.read().await;
        let albums = Self::build_albums(&tracks);
        *self.albums_cache.write().await = Some(albums.clone());
        albums
    }

    /// Get all artists with their albums (cached).
    pub async fn get_artists(&self) -> Vec<Artist> {
        {
            let cache = self.artists_cache.read().await;
            if let Some(ref artists) = *cache {
                return artists.clone();
            }
        }

        let albums = self.get_albums().await;
        let artists = Self::build_artists(&albums);
        *self.artists_cache.write().await = Some(artists.clone());
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
        let total_plays = tracks.iter().map(|t| t.play_count).sum();

        LibraryStats {
            total_tracks: tracks.len(),
            total_albums: albums.len(),
            total_artists: artists.len(),
            total_duration_secs,
            total_size_bytes,
            total_plays,
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
            .await
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
        self.invalidate_cache().await;

        tracing::info!(
            "Updated metadata for track: {} ({})",
            updated_track.title.as_deref().unwrap_or("Unknown"),
            id
        );

        Ok(updated_track)
    }

    /// Write metadata to an audio file (FLAC or MP3)
    async fn write_audio_metadata(&self, path: &Path, update: &TrackMetadataUpdate) -> Result<()> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("No file extension"))?;

        tracing::debug!("Writing metadata to {} file: {}", ext, path.display());

        let handler = get_audio_file_handler(ext)
            .ok_or_else(|| anyhow::anyhow!("Unsupported file format: {}", ext))?;

        let path_owned = path.to_path_buf();
        let update_owned = update.clone();
        tokio::task::spawn_blocking(move || {
            handler
                .write_metadata(&path_owned, &update_owned)
                .context(format!("Failed to write metadata to {}", path_owned.display()))
        })
        .await?
    }

    /// Get cover art from an audio file (FLAC or MP3)
    pub async fn get_cover_art(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("No file extension"))?;

        let handler = get_audio_file_handler(ext)
            .ok_or_else(|| anyhow::anyhow!("Unsupported file format: {}", ext))?;

        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || handler.get_cover_art(&path_owned)).await?
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

        let path_owned = track.path.clone();
        let mime_type_owned = mime_type.to_string();
        tokio::task::spawn_blocking(move || {
            handler.set_cover_art(&path_owned, image_data, &mime_type_owned)
        })
        .await??;

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
        self.invalidate_cache().await;

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

        let path_owned = track.path.clone();
        tokio::task::spawn_blocking(move || handler.remove_cover_art(&path_owned)).await??;

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
        self.invalidate_cache().await;

        tracing::info!("Removed cover art for track: {}", id);

        Ok(())
    }
}
