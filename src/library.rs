use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use symphonia::core::meta::MetadataOptions;

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
                        tracing::info!("Found track: {}", track.title.as_deref().unwrap_or("Unknown"));
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
        let file = std::fs::File::open(path)
            .context("Failed to open file")?;
        
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
        self.tracks.read().await
            .iter()
            .find(|t| t.id == id)
            .cloned()
    }

    /// Get the library path
    pub fn library_path(&self) -> &Path {
        &self.library_path
    }
}
