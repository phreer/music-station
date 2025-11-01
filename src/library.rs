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
    pub has_lyrics: bool,
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
                        if ext == "flac" || ext == "mp3" {
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
        let file = std::fs::File::open(path).context("Failed to open file")?;

        let metadata = tokio::fs::metadata(path).await?;
        let file_size = metadata.len();

        // Create a media source stream
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // Create a hint to help the format registry
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            hint.with_extension(ext);
        }

        // Probe the media source
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &Default::default(), &MetadataOptions::default())
            .context("Failed to probe file")?;

        let mut format = probed.format;
        let mut metadata = probed.metadata;

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

        // Standard tags to extract (supporting both FLAC/Vorbis and ID3 naming)
        let standard_tags = [
            "TITLE",
            "TIT2", // Title
            "ARTIST",
            "TPE1", // Artist
            "ALBUM",
            "TALB", // Album
            "ALBUMARTIST",
            "TPE2", // Album Artist
            "GENRE",
            "TCON", // Genre
            "DATE",
            "YEAR",
            "TDRC", // Year/Date
            "TRACKNUMBER",
            "TRCK", // Track Number
            "DISCNUMBER",
            "TPOS", // Disc Number
            "COMPOSER",
            "TCOM", // Composer
            "COMMENT",
            "COMM",
            "DESCRIPTION", // Comment
        ];

        let format_metadata = format.metadata();
        // Get metadata from format or metadata revisions
        if let Some(metadata_rev) = format_metadata.current().map_or_else(
            || {
                metadata
                    .get()
                    .map_or_else(|| None, |m| m.current().cloned())
            },
            |x| Some(x).cloned(),
        ) {
            for tag in metadata_rev.tags() {
                let key = tag.key.as_str();
                let value = tag.value.to_string();

                match key {
                    // Title (FLAC: TITLE, MP3: TIT2)
                    "TITLE" | "TIT2" => title = Some(value),
                    // Artist (FLAC: ARTIST, MP3: TPE1)
                    "ARTIST" | "TPE1" => artist = Some(value),
                    // Album (FLAC: ALBUM, MP3: TALB)
                    "ALBUM" | "TALB" => album = Some(value),
                    // Album Artist (FLAC: ALBUMARTIST, MP3: TPE2)
                    "ALBUMARTIST" | "TPE2" => album_artist = Some(value),
                    // Genre (FLAC: GENRE, MP3: TCON)
                    "GENRE" | "TCON" => genre = Some(value),
                    // Year (FLAC: DATE/YEAR, MP3: TDRC)
                    "DATE" | "YEAR" | "TDRC" => year = Some(value),
                    // Track Number (FLAC: TRACKNUMBER, MP3: TRCK)
                    "TRACKNUMBER" | "TRCK" => track_number = Some(value),
                    // Disc Number (FLAC: DISCNUMBER, MP3: TPOS)
                    "DISCNUMBER" | "TPOS" => disc_number = Some(value),
                    // Composer (FLAC: COMPOSER, MP3: TCOM)
                    "COMPOSER" | "TCOM" => composer = Some(value),
                    // Comment (FLAC: COMMENT/DESCRIPTION, MP3: COMM)
                    "COMMENT" | "COMM" | "DESCRIPTION" => comment = Some(value),
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
            has_lyrics: false, // Will be updated when lyrics database is queried
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

        match ext {
            "flac" => self.write_flac_metadata(path, update).context(format!(
                "Failed to write FLAC metadata to {}",
                path.display()
            )),
            "mp3" => self.write_mp3_metadata(path, update).context(format!(
                "Failed to write MP3 metadata to {}",
                path.display()
            )),
            _ => anyhow::bail!("Unsupported file format: {}", ext),
        }
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

    /// Write metadata to an MP3 file
    fn write_mp3_metadata(&self, path: &Path, update: &TrackMetadataUpdate) -> Result<()> {
        use id3::TagLike;

        tracing::debug!("Reading MP3 tags from: {}", path.display());

        let mut tag = id3::Tag::read_from_path(path)
            .or_else(|e| {
                tracing::warn!("Failed to read existing MP3 tags ({}), creating new tag", e);
                Ok::<_, anyhow::Error>(id3::Tag::new())
            })
            .context("Failed to read MP3 tags")?;

        // Update ID3v2 frames using the TagLike trait
        if let Some(title) = &update.title {
            tracing::debug!("Setting title: {}", title);
            tag.set_title(title);
        }
        if let Some(artist) = &update.artist {
            tracing::debug!("Setting artist: {}", artist);
            tag.set_artist(artist);
        }
        if let Some(album) = &update.album {
            tracing::debug!("Setting album: {}", album);
            tag.set_album(album);
        }
        if let Some(album_artist) = &update.album_artist {
            tracing::debug!("Setting album artist: {}", album_artist);
            tag.set_album_artist(album_artist);
        }
        if let Some(genre) = &update.genre {
            tracing::debug!("Setting genre: {}", genre);
            tag.set_genre(genre);
        }
        if let Some(year) = &update.year {
            if let Ok(year_num) = year.parse::<i32>() {
                tracing::debug!("Setting year: {}", year_num);
                tag.set_year(year_num);
            } else {
                tracing::warn!("Invalid year format: {}", year);
            }
        }
        if let Some(track_number) = &update.track_number {
            if let Ok(track_num) = track_number.parse::<u32>() {
                tracing::debug!("Setting track number: {}", track_num);
                tag.set_track(track_num);
            } else {
                tracing::warn!("Invalid track number format: {}", track_number);
            }
        }
        if let Some(disc_number) = &update.disc_number {
            if let Ok(disc_num) = disc_number.parse::<u32>() {
                tracing::debug!("Setting disc number: {}", disc_num);
                tag.set_disc(disc_num);
            } else {
                tracing::warn!("Invalid disc number format: {}", disc_number);
            }
        }
        // Note: composer and comment require adding frames directly

        tracing::debug!("Writing MP3 tags to file: {}", path.display());

        // Check file permissions before writing
        let metadata = std::fs::metadata(path).context(format!(
            "Failed to read file metadata for {}",
            path.display()
        ))?;

        if metadata.permissions().readonly() {
            anyhow::bail!("File is read-only: {}", path.display());
        }

        tag.write_to_path(path, id3::Version::Id3v23).map_err(|e| {
            tracing::error!("ID3 write error details: {:?}", e);
            anyhow::anyhow!("Failed to save MP3 tags to {}: {}", path.display(), e)
        })?;

        tracing::debug!("Successfully wrote MP3 metadata to: {}", path.display());

        Ok(())
    }

    /// Check if an audio file has embedded cover art
    fn has_embedded_cover(&self, path: &Path) -> bool {
        let ext = path.extension().and_then(|s| s.to_str());

        match ext {
            Some("flac") => {
                if let Ok(tag) = metaflac::Tag::read_from_path(path) {
                    tag.pictures().count() > 0
                } else {
                    false
                }
            }
            Some("mp3") => {
                if let Ok(tag) = id3::Tag::read_from_path(path) {
                    tag.pictures().count() > 0
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Get cover art from an audio file (FLAC or MP3)
    pub fn get_cover_art(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("No file extension"))?;

        match ext {
            "flac" => {
                let tag =
                    metaflac::Tag::read_from_path(path).context("Failed to read FLAC tags")?;
                // Get the first picture (usually the front cover)
                if let Some(picture) = tag.pictures().next() {
                    Ok(Some(picture.data.clone()))
                } else {
                    Ok(None)
                }
            }
            "mp3" => {
                let tag = id3::Tag::read_from_path(path).context("Failed to read MP3 tags")?;
                // Get the first picture
                if let Some(picture) = tag.pictures().next() {
                    Ok(Some(picture.data.to_vec()))
                } else {
                    Ok(None)
                }
            }
            _ => anyhow::bail!("Unsupported file format: {}", ext),
        }
    }

    /// Set cover art for an audio file (FLAC or MP3)
    pub async fn set_cover_art(
        &self,
        id: &str,
        image_data: Vec<u8>,
        mime_type: &str,
    ) -> Result<()> {
        use id3::TagLike;

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

        match ext {
            "flac" => {
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
                tag.add_picture(picture.mime_type, picture.picture_type, picture.data);
                tag.save()
                    .context("Failed to save FLAC tags with cover art")?;
            }
            "mp3" => {
                let mut tag = id3::Tag::read_from_path(&track.path)
                    .or_else(|_| Ok::<_, anyhow::Error>(id3::Tag::new()))
                    .context("Failed to read MP3 tags")?;

                // Remove existing pictures
                tag.remove_all_pictures();

                // Add new picture
                let picture = id3::frame::Picture {
                    mime_type: mime_type.to_string(),
                    picture_type: id3::frame::PictureType::CoverFront,
                    description: String::new(),
                    data: image_data,
                };
                tag.add_frame(picture);

                tag.write_to_path(&track.path, id3::Version::Id3v24)
                    .context("Failed to save MP3 tags with cover art")?;
            }
            _ => anyhow::bail!("Unsupported file format: {}", ext),
        }

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
        use id3::TagLike;

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

        match ext {
            "flac" => {
                let mut tag = metaflac::Tag::read_from_path(&track.path)
                    .context("Failed to read FLAC tags")?;

                // Remove all pictures
                tag.remove_picture_type(metaflac::block::PictureType::CoverFront);
                tag.save().context("Failed to save FLAC tags")?;
            }
            "mp3" => {
                let mut tag =
                    id3::Tag::read_from_path(&track.path).context("Failed to read MP3 tags")?;

                // Remove all pictures
                tag.remove_all_pictures();

                tag.write_to_path(&track.path, id3::Version::Id3v24)
                    .context("Failed to save MP3 tags")?;
            }
            _ => anyhow::bail!("Unsupported file format: {}", ext),
        }

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
