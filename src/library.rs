use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    pub custom_fields: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct TrackMetadataUpdate {
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
    pub custom_fields: Option<HashMap<String, String>>,
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
        let mut album_artist = None;
        let mut genre = None;
        let mut year = None;
        let mut track_number = None;
        let mut disc_number = None;
        let mut composer = None;
        let mut comment = None;
        let mut duration_secs = None;
        let mut custom_fields = HashMap::new();

        // Standard FLAC/Vorbis comment tags to extract
        let standard_tags = [
            "TITLE", "ARTIST", "ALBUM", "ALBUMARTIST", "GENRE", "DATE", 
            "TRACKNUMBER", "DISCNUMBER", "COMPOSER", "COMMENT", "DESCRIPTION"
        ];

        // Get metadata from format or metadata revisions
        if let Some(metadata_rev) = format.metadata().current() {
            for tag in metadata_rev.tags() {
                let key = tag.key.as_str();
                let value = tag.value.to_string();

                match key {
                    "TITLE" => title = Some(value),
                    "ARTIST" => artist = Some(value),
                    "ALBUM" => album = Some(value),
                    "ALBUMARTIST" => album_artist = Some(value),
                    "GENRE" => genre = Some(value),
                    "DATE" | "YEAR" => year = Some(value),
                    "TRACKNUMBER" => track_number = Some(value),
                    "DISCNUMBER" => disc_number = Some(value),
                    "COMPOSER" => composer = Some(value),
                    "COMMENT" | "DESCRIPTION" => comment = Some(value),
                    // Store any other tags as custom fields
                    _ => {
                        if !standard_tags.contains(&key) {
                            custom_fields.insert(key.to_string(), value);
                        }
                    }
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

        // Check for embedded cover art
        let has_cover = self.has_embedded_cover(path);

        // Generate a unique ID from the file path
        let id = format!("{:x}", md5::compute(path.to_string_lossy().as_bytes()));

        Ok(Track {
            id,
            path: path.to_path_buf(),
            title,
            artist,
            album,
            album_artist,
            genre,
            year,
            track_number,
            disc_number,
            composer,
            comment,
            duration_secs,
            file_size,
            has_cover,
            custom_fields,
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

        // Update standard vorbis comments
        if let Some(title) = &update.title {
            tag.set_vorbis("TITLE", vec![title.clone()]);
        }
        if let Some(artist) = &update.artist {
            tag.set_vorbis("ARTIST", vec![artist.clone()]);
        }
        if let Some(album) = &update.album {
            tag.set_vorbis("ALBUM", vec![album.clone()]);
        }
        if let Some(album_artist) = &update.album_artist {
            tag.set_vorbis("ALBUMARTIST", vec![album_artist.clone()]);
        }
        if let Some(genre) = &update.genre {
            tag.set_vorbis("GENRE", vec![genre.clone()]);
        }
        if let Some(year) = &update.year {
            tag.set_vorbis("DATE", vec![year.clone()]);
        }
        if let Some(track_number) = &update.track_number {
            tag.set_vorbis("TRACKNUMBER", vec![track_number.clone()]);
        }
        if let Some(disc_number) = &update.disc_number {
            tag.set_vorbis("DISCNUMBER", vec![disc_number.clone()]);
        }
        if let Some(composer) = &update.composer {
            tag.set_vorbis("COMPOSER", vec![composer.clone()]);
        }
        if let Some(comment) = &update.comment {
            tag.set_vorbis("COMMENT", vec![comment.clone()]);
        }

        // Update custom fields
        if let Some(custom_fields) = &update.custom_fields {
            for (key, value) in custom_fields {
                tag.set_vorbis(key, vec![value.clone()]);
            }
        }

        tag.save().context("Failed to save FLAC tags")?;

        Ok(())
    }

    /// Check if a FLAC file has embedded cover art
    fn has_embedded_cover(&self, path: &Path) -> bool {
        if let Ok(tag) = metaflac::Tag::read_from_path(path) {
            tag.pictures().count() > 0
        } else {
            false
        }
    }

    /// Get cover art from a FLAC file
    pub fn get_cover_art(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        let tag = metaflac::Tag::read_from_path(path).context("Failed to read FLAC tags")?;

        // Get the first picture (usually the front cover)
        if let Some(picture) = tag.pictures().next() {
            Ok(Some(picture.data.clone()))
        } else {
            Ok(None)
        }
    }

    /// Set cover art for a FLAC file
    pub async fn set_cover_art(&self, id: &str, image_data: Vec<u8>, mime_type: &str) -> Result<()> {
        // Find the track
        let track = {
            let tracks = self.tracks.read().await;
            tracks
                .iter()
                .find(|t| t.id == id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Track not found"))?
        };

        let mut tag = metaflac::Tag::read_from_path(&track.path)
            .context("Failed to read FLAC tags")?;

        // Remove existing pictures
        tag.remove_picture_type(metaflac::block::PictureType::CoverFront);

        // Create new picture block
        let picture = metaflac::block::Picture {
            picture_type: metaflac::block::PictureType::CoverFront,
            mime_type: mime_type.to_string(),
            description: String::new(),
            width: 0,
            height: 0,
            depth: 0,
            num_colors: 0,
            data: image_data,
        };

        // Add picture to tag
        tag.add_picture(
            picture.mime_type,
            picture.picture_type,
            picture.data,
        );
        tag.save().context("Failed to save FLAC tags with cover art")?;

        // Update in-memory track
        let updated_track = self
            .parse_flac_file(&track.path)
            .await
            .context("Failed to re-parse file after cover update")?;

        {
            let mut tracks = self.tracks.write().await;
            if let Some(pos) = tracks.iter().position(|t| t.id == id) {
                tracks[pos] = updated_track;
            }
        }

        tracing::info!("Updated cover art for track: {}", id);

        Ok(())
    }

    /// Remove cover art from a FLAC file
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

        let mut tag = metaflac::Tag::read_from_path(&track.path)
            .context("Failed to read FLAC tags")?;

        // Remove all pictures
        tag.remove_picture_type(metaflac::block::PictureType::CoverFront);
        tag.save().context("Failed to save FLAC tags")?;

        // Update in-memory track
        let updated_track = self
            .parse_flac_file(&track.path)
            .await
            .context("Failed to re-parse file after cover removal")?;

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
