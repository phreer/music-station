use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub path: PathBuf,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_secs: Option<u64>,
    pub file_size: u64,
}

#[derive(Debug, Deserialize)]
pub struct TrackMetadataUpdate {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

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

    /// Scan the library folder for FLAC files
    pub async fn scan(&self) -> Result<()> {
        tracing::info!("Scanning library at: {}", self.library_path.display());

        let mut tracks = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.library_path)
            .await
            .context("Failed to read library directory")?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Only process FLAC files
            if path.extension().and_then(|s| s.to_str()) == Some("flac") {
                match self.parse_flac_file(&path).await {
                    Ok(track) => {
                        tracing::info!(
                            "Found track: {}",
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

        let mut library_tracks = self.tracks.write().await;
        *library_tracks = tracks;

        tracing::info!("Scan complete. Found {} tracks", library_tracks.len());
        Ok(())
    }

    /// Parse a FLAC file and extract metadata
    async fn parse_flac_file(&self, path: &Path) -> Result<Track> {
        let file = std::fs::File::open(path).context("Failed to open file")?;

        let metadata = tokio::fs::metadata(path).await?;
        let file_size = metadata.len();

        // Create a media source stream
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // Create a hint to help the format registry
        let mut hint = Hint::new();
        hint.with_extension("flac");

        // Probe the media source
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &Default::default(), &MetadataOptions::default())
            .context("Failed to probe file")?;

        let mut format = probed.format;

        // Extract metadata
        let mut title = None;
        let mut artist = None;
        let mut album = None;
        let mut duration_secs = None;

        // Get metadata from format or metadata revisions
        if let Some(metadata_rev) = format.metadata().current() {
            for tag in metadata_rev.tags() {
                match tag.key.as_str() {
                    "TITLE" => title = Some(tag.value.to_string()),
                    "ARTIST" => artist = Some(tag.value.to_string()),
                    "ALBUM" => album = Some(tag.value.to_string()),
                    _ => {}
                }
            }
        }

        // Get duration from the default track
        if let Some(track) = format.default_track() {
            if let Some(time_base) = track.codec_params.time_base {
                if let Some(n_frames) = track.codec_params.n_frames {
                    duration_secs = Some(time_base.calc_time(n_frames).seconds);
                }
            }
        }

        // Generate a unique ID from the file path
        let id = format!("{:x}", md5::compute(path.to_string_lossy().as_bytes()));

        Ok(Track {
            id,
            path: path.to_path_buf(),
            title,
            artist,
            album,
            duration_secs,
            file_size,
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
                .ok_or_else(|| anyhow::anyhow!("Track not found"))?
        };

        // Update the FLAC file metadata
        self.write_flac_metadata(&track.path, &update)
            .context("Failed to write metadata to file")?;

        // Re-parse the file to get updated metadata
        let updated_track = self
            .parse_flac_file(&track.path)
            .await
            .context("Failed to re-parse file after update")?;

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

    /// Write metadata to a FLAC file
    fn write_flac_metadata(&self, path: &Path, update: &TrackMetadataUpdate) -> Result<()> {
        let mut tag = metaflac::Tag::read_from_path(path).context("Failed to read FLAC tags")?;

        // Update vorbis comments
        if let Some(title) = &update.title {
            tag.set_vorbis("TITLE", vec![title.clone()]);
        }
        if let Some(artist) = &update.artist {
            tag.set_vorbis("ARTIST", vec![artist.clone()]);
        }
        if let Some(album) = &update.album {
            tag.set_vorbis("ALBUM", vec![album.clone()]);
        }

        tag.save().context("Failed to save FLAC tags")?;

        Ok(())
    }
}
