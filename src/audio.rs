use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Metadata update request for an audio file
#[derive(Debug, Clone, Deserialize)]
pub struct MetadataUpdate {
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

/// Metadata extracted from an audio file
#[derive(Debug, Clone, Serialize)]
pub struct AudioMetadata {
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
    pub custom_fields: HashMap<String, String>,
}

impl AudioMetadata {
    pub fn new() -> Self {
        AudioMetadata {
            title: None,
            artist: None,
            album: None,
            album_artist: None,
            genre: None,
            year: None,
            track_number: None,
            disc_number: None,
            composer: None,
            comment: None,
            duration_secs: None,
            custom_fields: HashMap::new(),
        }
    }
    pub fn update_from_std_key(
        &mut self,
        std_key: symphonia::core::meta::StandardTagKey,
        value: String,
    ) {
        match std_key {
            symphonia::core::meta::StandardTagKey::TrackTitle => self.title = Some(value),
            symphonia::core::meta::StandardTagKey::Artist => self.artist = Some(value),
            symphonia::core::meta::StandardTagKey::Album => self.album = Some(value),
            symphonia::core::meta::StandardTagKey::AlbumArtist => self.album_artist = Some(value),
            symphonia::core::meta::StandardTagKey::Genre => self.genre = Some(value),
            symphonia::core::meta::StandardTagKey::Date => self.year = Some(value),
            symphonia::core::meta::StandardTagKey::TrackNumber => self.track_number = Some(value),
            symphonia::core::meta::StandardTagKey::DiscNumber => self.disc_number = Some(value),
            symphonia::core::meta::StandardTagKey::Composer => self.composer = Some(value),
            symphonia::core::meta::StandardTagKey::Comment => self.comment = Some(value),
            _ => {}
        }
    }
}
/// Trait representing operations on audio files
pub trait AudioFile: Send + Sync {
    /// Get the file format name (e.g., "flac", "mp3")
    fn format_name(&self) -> &'static str;

    /// Parse metadata from the audio file
    fn parse_metadata(&self, path: &Path) -> Result<AudioMetadata>;

    /// Write metadata to the audio file
    fn write_metadata(&self, path: &Path, update: &MetadataUpdate) -> Result<()>;

    /// Check if the file has embedded cover art
    fn has_cover_art(&self, path: &Path) -> Result<bool>;

    /// Get cover art data from the file
    fn get_cover_art(&self, path: &Path) -> Result<Option<Vec<u8>>>;

    /// Set cover art for the file
    fn set_cover_art(&self, path: &Path, data: Vec<u8>, mime_type: &str) -> Result<()>;

    /// Remove cover art from the file
    fn remove_cover_art(&self, path: &Path) -> Result<()>;
}

/// FLAC audio file implementation
pub struct FlacFile;

impl AudioFile for FlacFile {
    fn format_name(&self) -> &'static str {
        "flac"
    }

    fn parse_metadata(&self, path: &Path) -> Result<AudioMetadata> {
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let file = std::fs::File::open(path).context("Failed to open FLAC file")?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        hint.with_extension("flac");

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &Default::default(), &MetadataOptions::default())
            .context("Failed to probe FLAC file")?;

        let mut format = probed.format;
        let mut metadata = probed.metadata;

        let mut audio_metadata = AudioMetadata::new();

        // Standard tags for FLAC (Vorbis comments)
        let standard_tags = [
            "TITLE",
            "ARTIST",
            "ALBUM",
            "ALBUMARTIST",
            "GENRE",
            "DATE",
            "YEAR",
            "TRACKNUMBER",
            "DISCNUMBER",
            "COMPOSER",
            "COMMENT",
            "DESCRIPTION",
        ];

        // Extract metadata from Vorbis comments
        let format_metadata = format.metadata();
        if let Some(metadata_rev) = format_metadata.current().map_or_else(
            || metadata.get().and_then(|m| m.current().cloned()),
            |x| Some(x).cloned(),
        ) {
            for tag in metadata_rev.tags() {
                let key = tag.key.to_uppercase();
                let value = tag.value.to_string();

                tracing::debug!("FLAC metadata tag: {} = {}", key, value);

                match key.as_str() {
                    "TITLE" => audio_metadata.title = Some(value),
                    "ARTIST" => audio_metadata.artist = Some(value),
                    "ALBUM" => audio_metadata.album = Some(value),
                    "ALBUMARTIST" => audio_metadata.album_artist = Some(value),
                    "GENRE" => audio_metadata.genre = Some(value),
                    "DATE" | "YEAR" => audio_metadata.year = Some(value),
                    "TRACKNUMBER" => audio_metadata.track_number = Some(value),
                    "DISCNUMBER" => audio_metadata.disc_number = Some(value),
                    "COMPOSER" => audio_metadata.composer = Some(value),
                    "COMMENT" | "DESCRIPTION" => audio_metadata.comment = Some(value),
                    _ => {
                        if !standard_tags.contains(&key.as_str()) {
                            audio_metadata.custom_fields.insert(key, value);
                        }
                    }
                }
            }
        }

        // Get duration from the default track
        if let Some(track) = format.default_track() {
            if let Some(time_base) = track.codec_params.time_base {
                if let Some(n_frames) = track.codec_params.n_frames {
                    audio_metadata.duration_secs = Some(time_base.calc_time(n_frames).seconds);
                }
            }
        }

        Ok(audio_metadata)
    }

    fn write_metadata(&self, path: &Path, update: &MetadataUpdate) -> Result<()> {
        let mut tag = metaflac::Tag::read_from_path(path).context("Failed to read FLAC tags")?;

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

        if let Some(custom_fields) = &update.custom_fields {
            for (key, value) in custom_fields {
                tag.set_vorbis(key, vec![value.clone()]);
            }
        }

        tag.save().context("Failed to save FLAC tags")?;
        Ok(())
    }

    fn has_cover_art(&self, path: &Path) -> Result<bool> {
        let tag = metaflac::Tag::read_from_path(path).context("Failed to read FLAC tags")?;
        Ok(tag.pictures().count() > 0)
    }

    fn get_cover_art(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        let tag = metaflac::Tag::read_from_path(path).context("Failed to read FLAC tags")?;
        if let Some(picture) = tag.pictures().next() {
            Ok(Some(picture.data.clone()))
        } else {
            Ok(None)
        }
    }

    fn set_cover_art(&self, path: &Path, data: Vec<u8>, mime_type: &str) -> Result<()> {
        let mut tag = metaflac::Tag::read_from_path(path).context("Failed to read FLAC tags")?;

        tag.remove_picture_type(metaflac::block::PictureType::CoverFront);

        let picture = metaflac::block::Picture {
            picture_type: metaflac::block::PictureType::CoverFront,
            mime_type: mime_type.to_string(),
            description: String::new(),
            width: 0,
            height: 0,
            depth: 0,
            num_colors: 0,
            data,
        };

        tag.add_picture(picture.mime_type, picture.picture_type, picture.data);
        tag.save()
            .context("Failed to save FLAC tags with cover art")?;
        Ok(())
    }

    fn remove_cover_art(&self, path: &Path) -> Result<()> {
        let mut tag = metaflac::Tag::read_from_path(path).context("Failed to read FLAC tags")?;
        tag.remove_picture_type(metaflac::block::PictureType::CoverFront);
        tag.save().context("Failed to save FLAC tags")?;
        Ok(())
    }
}

/// MP3 audio file implementation
pub struct Mp3File;

impl AudioFile for Mp3File {
    fn format_name(&self) -> &'static str {
        "mp3"
    }

    fn parse_metadata(&self, path: &Path) -> Result<AudioMetadata> {
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let file = std::fs::File::open(path).context("Failed to open MP3 file")?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        hint.with_extension("mp3");

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &Default::default(), &MetadataOptions::default())
            .context("Failed to probe MP3 file")?;

        let mut format = probed.format;
        let mut metadata = probed.metadata;

        let mut audio_metadata = AudioMetadata::new();

        // Standard tags for MP3 (ID3v2)
        let standard_tags = [
            "TIT2", "TPE1", "TALB", "TPE2", "TCON", "TDRC", "TRCK", "TPOS", "TCOM", "COMM",
        ];

        // Extract metadata from ID3 tags
        let format_metadata = format.metadata();
        if let Some(metadata_rev) = format_metadata.current().map_or_else(
            || metadata.get().and_then(|m| m.current().cloned()),
            |x| Some(x).cloned(),
        ) {
            for tag in metadata_rev.tags() {
                let key = tag.key.to_uppercase();
                let value = tag.value.to_string();

                tracing::debug!("MP3 metadata tag: {} = {}", key, value);

                match key.as_str() {
                    "TIT2" => audio_metadata.title = Some(value),
                    "TPE1" => audio_metadata.artist = Some(value),
                    "TALB" => audio_metadata.album = Some(value),
                    "TPE2" => audio_metadata.album_artist = Some(value),
                    "TCON" => audio_metadata.genre = Some(value),
                    "TDRC" => audio_metadata.year = Some(value),
                    "TRCK" => audio_metadata.track_number = Some(value),
                    "TPOS" => audio_metadata.disc_number = Some(value),
                    "TCOM" => audio_metadata.composer = Some(value),
                    "COMM" => audio_metadata.comment = Some(value),
                    _ => {
                        if !standard_tags.contains(&key.as_str()) {
                            audio_metadata.custom_fields.insert(key, value);
                        }
                    }
                }
            }
        }

        // Get duration from the default track
        if let Some(track) = format.default_track() {
            if let Some(time_base) = track.codec_params.time_base {
                if let Some(n_frames) = track.codec_params.n_frames {
                    audio_metadata.duration_secs = Some(time_base.calc_time(n_frames).seconds);
                }
            }
        }

        Ok(audio_metadata)
    }

    fn write_metadata(&self, path: &Path, update: &MetadataUpdate) -> Result<()> {
        use id3::TagLike;

        tracing::debug!("Reading MP3 tags from: {}", path.display());

        let mut tag = id3::Tag::read_from_path(path).or_else(|e| {
            tracing::warn!("Failed to read existing MP3 tags ({}), creating new tag", e);
            Ok::<_, anyhow::Error>(id3::Tag::new())
        })?;

        if let Some(title) = &update.title {
            tag.set_title(title);
        }
        if let Some(artist) = &update.artist {
            tag.set_artist(artist);
        }
        if let Some(album) = &update.album {
            tag.set_album(album);
        }
        if let Some(album_artist) = &update.album_artist {
            tag.set_album_artist(album_artist);
        }
        if let Some(genre) = &update.genre {
            tag.set_genre(genre);
        }
        if let Some(year) = &update.year {
            if let Ok(year_num) = year.parse::<i32>() {
                tag.set_year(year_num);
            } else {
                tracing::warn!("Invalid year format: {}", year);
            }
        }
        if let Some(track_number) = &update.track_number {
            if let Ok(track_num) = track_number.parse::<u32>() {
                tag.set_track(track_num);
            } else {
                tracing::warn!("Invalid track number format: {}", track_number);
            }
        }
        if let Some(disc_number) = &update.disc_number {
            if let Ok(disc_num) = disc_number.parse::<u32>() {
                tag.set_disc(disc_num);
            } else {
                tracing::warn!("Invalid disc number format: {}", disc_number);
            }
        }

        tracing::debug!("Writing MP3 tags to file: {}", path.display());

        let metadata = std::fs::metadata(path).context("Failed to read file metadata")?;
        if metadata.permissions().readonly() {
            anyhow::bail!("File is read-only: {}", path.display());
        }

        tag.write_to_path(path, id3::Version::Id3v23)
            .context("Failed to save MP3 tags")?;

        Ok(())
    }

    fn has_cover_art(&self, path: &Path) -> Result<bool> {
        let tag = id3::Tag::read_from_path(path).context("Failed to read MP3 tags")?;
        Ok(tag.pictures().count() > 0)
    }

    fn get_cover_art(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        let tag = id3::Tag::read_from_path(path).context("Failed to read MP3 tags")?;
        if let Some(picture) = tag.pictures().next() {
            Ok(Some(picture.data.to_vec()))
        } else {
            Ok(None)
        }
    }

    fn set_cover_art(&self, path: &Path, data: Vec<u8>, mime_type: &str) -> Result<()> {
        use id3::TagLike;

        let mut tag =
            id3::Tag::read_from_path(path).or_else(|_| Ok::<_, anyhow::Error>(id3::Tag::new()))?;

        tag.remove_all_pictures();

        let picture = id3::frame::Picture {
            mime_type: mime_type.to_string(),
            picture_type: id3::frame::PictureType::CoverFront,
            description: String::new(),
            data,
        };
        tag.add_frame(picture);

        tag.write_to_path(path, id3::Version::Id3v24)
            .context("Failed to save MP3 tags with cover art")?;
        Ok(())
    }

    fn remove_cover_art(&self, path: &Path) -> Result<()> {
        use id3::TagLike;

        let mut tag = id3::Tag::read_from_path(path).context("Failed to read MP3 tags")?;
        tag.remove_all_pictures();
        tag.write_to_path(path, id3::Version::Id3v24)
            .context("Failed to save MP3 tags")?;
        Ok(())
    }
}

/// OGG Vorbis audio file implementation
pub struct OggFile;

impl AudioFile for OggFile {
    fn format_name(&self) -> &'static str {
        "ogg"
    }

    fn parse_metadata(&self, path: &Path) -> Result<AudioMetadata> {
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let file = std::fs::File::open(path).context("Failed to open OGG file")?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        hint.with_extension("ogg");

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &Default::default(), &MetadataOptions::default())
            .context("Failed to probe OGG file")?;

        let mut format = probed.format;
        let mut metadata = probed.metadata;

        let mut audio_metadata = AudioMetadata::new();

        // Standard tags for OGG Vorbis (Vorbis comments - same as FLAC)
        let standard_tags = [
            "TITLE",
            "ARTIST",
            "ALBUM",
            "ALBUMARTIST",
            "GENRE",
            "DATE",
            "YEAR",
            "TRACKNUMBER",
            "DISCNUMBER",
            "COMPOSER",
            "COMMENT",
            "DESCRIPTION",
        ];

        // Extract metadata from Vorbis comments
        let format_metadata = format.metadata();
        if let Some(metadata_rev) = format_metadata.current().map_or_else(
            || metadata.get().and_then(|m| m.current().cloned()),
            |x| Some(x).cloned(),
        ) {
            for tag in metadata_rev.tags() {
                let key = tag.key.to_uppercase();
                let value = tag.value.to_string();

                tracing::debug!("OGG metadata tag: {} = {}", key, value);

                match key.as_str() {
                    "TITLE" => audio_metadata.title = Some(value),
                    "ARTIST" => audio_metadata.artist = Some(value),
                    "ALBUM" => audio_metadata.album = Some(value),
                    "ALBUMARTIST" => audio_metadata.album_artist = Some(value),
                    "GENRE" => audio_metadata.genre = Some(value),
                    "DATE" | "YEAR" => audio_metadata.year = Some(value),
                    "TRACKNUMBER" => audio_metadata.track_number = Some(value),
                    "DISCNUMBER" => audio_metadata.disc_number = Some(value),
                    "COMPOSER" => audio_metadata.composer = Some(value),
                    "COMMENT" | "DESCRIPTION" => audio_metadata.comment = Some(value),
                    _ => {
                        if !standard_tags.contains(&key.as_str()) {
                            audio_metadata.custom_fields.insert(key, value);
                        }
                    }
                }
            }
        }

        // Get duration from the default track
        if let Some(track) = format.default_track() {
            if let Some(time_base) = track.codec_params.time_base {
                if let Some(n_frames) = track.codec_params.n_frames {
                    audio_metadata.duration_secs = Some(time_base.calc_time(n_frames).seconds);
                }
            }
        }

        Ok(audio_metadata)
    }

    fn write_metadata(&self, path: &Path, update: &MetadataUpdate) -> Result<()> {
        // OGG Vorbis metadata writing requires external tools or specialized libraries
        // For now, we'll return an error indicating this is not yet supported
        // TODO: Implement OGG metadata writing using vorbis-comments or similar crate
        anyhow::bail!(
            "OGG metadata writing is not yet supported. File: {}",
            path.display()
        )
    }

    fn has_cover_art(&self, path: &Path) -> Result<bool> {
        // OGG Vorbis can have embedded artwork through METADATA_BLOCK_PICTURE
        // We'll check this through Symphonia's metadata
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let file = std::fs::File::open(path).context("Failed to open OGG file")?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        hint.with_extension("ogg");

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &Default::default(), &MetadataOptions::default())
            .context("Failed to probe OGG file")?;

        let mut format = probed.format;
        let mut metadata = probed.metadata;

        // Check for visual metadata (cover art)
        let format_metadata = format.metadata();
        if let Some(metadata_rev) = format_metadata.current().map_or_else(
            || metadata.get().and_then(|m| m.current().cloned()),
            |x| Some(x).cloned(),
        ) {
            Ok(metadata_rev.visuals().len() > 0)
        } else {
            Ok(false)
        }
    }

    fn get_cover_art(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let file = std::fs::File::open(path).context("Failed to open OGG file")?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        hint.with_extension("ogg");

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &Default::default(), &MetadataOptions::default())
            .context("Failed to probe OGG file")?;

        let mut format = probed.format;
        let mut metadata = probed.metadata;

        // Get visual metadata (cover art)
        let format_metadata = format.metadata();
        if let Some(metadata_rev) = format_metadata.current().map_or_else(
            || metadata.get().and_then(|m| m.current().cloned()),
            |x| Some(x).cloned(),
        ) {
            if let Some(visual) = metadata_rev.visuals().first() {
                Ok(Some(visual.data.to_vec()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn set_cover_art(&self, path: &Path, data: Vec<u8>, mime_type: &str) -> Result<()> {
        // OGG cover art writing is not yet supported
        // TODO: Implement using a suitable library
        anyhow::bail!(
            "OGG cover art writing is not yet supported. File: {}",
            path.display()
        )
    }

    fn remove_cover_art(&self, path: &Path) -> Result<()> {
        // OGG cover art removal is not yet supported
        // TODO: Implement using a suitable library
        anyhow::bail!(
            "OGG cover art removal is not yet supported. File: {}",
            path.display()
        )
    }
}

/// M4A (AAC) audio file implementation
pub struct M4aFile;

impl AudioFile for M4aFile {
    fn format_name(&self) -> &'static str {
        "m4a"
    }

    fn parse_metadata(&self, path: &Path) -> Result<AudioMetadata> {
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let file = std::fs::File::open(path).context("Failed to open M4A file")?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        hint.with_extension("m4a");

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &Default::default(), &MetadataOptions::default())
            .context("Failed to probe M4A file")?;

        let mut format = probed.format;
        let mut metadata = probed.metadata;

        let mut audio_metadata = AudioMetadata::new();

        // Standard tags for M4A (iTunes-style tags)
        let standard_tags = [
            "©NAM", // title
            "©ART", // artist
            "©ALB", // album
            "AART", // album artist
            "©GEN", // genre
            "©DAY", // year
            "TRKN", // track number
            "DISK", // disc number
            "©WRT", // composer
            "©CMT", // comment
        ];

        // Extract metadata from M4A tags
        let format_metadata = format.metadata();
        if let Some(metadata_rev) = format_metadata.current().map_or_else(
            || metadata.get().and_then(|m| m.current().cloned()),
            |x| Some(x).cloned(),
        ) {
            tracing::debug!("M4A metadata revision found: {:?}", metadata_rev);
            for tag in metadata_rev.tags() {
                let key = &tag.key;
                let value = tag.value.to_string();

                if let Some(std_key) = tag.std_key {
                    audio_metadata.update_from_std_key(std_key, value);
                } else {
                    tracing::debug!("M4A custom metadata tag: {} = {}", key, value);
                    match key.as_str() {
                        "©NAM" | "NAME" => audio_metadata.title = Some(value),
                        "©ART" | "ARTIST" => audio_metadata.artist = Some(value),
                        "©ALB" | "ALBUM" => audio_metadata.album = Some(value),
                        "AART" | "ALBUMARTIST" => audio_metadata.album_artist = Some(value),
                        "©GEN" | "GENRE" => audio_metadata.genre = Some(value),
                        "©DAY" | "DATE" | "YEAR" => audio_metadata.year = Some(value),
                        "TRKN" | "TRACKNUMBER" => audio_metadata.track_number = Some(value),
                        "DISK" | "DISCNUMBER" => audio_metadata.disc_number = Some(value),
                        "©WRT" | "COMPOSER" => audio_metadata.composer = Some(value),
                        "©CMT" | "COMMENT" => audio_metadata.comment = Some(value),
                        _ => {
                            if !standard_tags.contains(&key.as_str()) {
                                audio_metadata.custom_fields.insert(key.to_string(), value);
                            }
                        }
                    }
                }
            }
        }

        // Get duration from the default track
        if let Some(track) = format.default_track() {
            if let Some(time_base) = track.codec_params.time_base {
                if let Some(n_frames) = track.codec_params.n_frames {
                    audio_metadata.duration_secs = Some(time_base.calc_time(n_frames).seconds);
                }
            }
        }

        Ok(audio_metadata)
    }

    fn write_metadata(&self, path: &Path, update: &MetadataUpdate) -> Result<()> {
        use mp4ameta::Tag;

        let mut tag =
            Tag::read_from_path(path).or_else(|_| Ok::<_, anyhow::Error>(Tag::default()))?;

        // Update basic metadata fields
        if let Some(ref title) = update.title {
            tag.set_title(title);
        }
        if let Some(ref artist) = update.artist {
            tag.set_artist(artist);
        }
        if let Some(ref album) = update.album {
            tag.set_album(album);
        }
        if let Some(ref album_artist) = update.album_artist {
            tag.set_album_artist(album_artist);
        }
        if let Some(ref genre) = update.genre {
            tag.set_genre(genre);
        }
        if let Some(ref year) = update.year {
            tag.set_year(year);
        }
        if let Some(ref track_number) = update.track_number {
            if let Ok(num) = track_number.parse::<u16>() {
                tag.set_track_number(num);
            }
        }
        if let Some(ref disc_number) = update.disc_number {
            if let Ok(num) = disc_number.parse::<u16>() {
                tag.set_disc_number(num);
            }
        }
        if let Some(ref composer) = update.composer {
            tag.set_composer(composer);
        }
        if let Some(ref comment) = update.comment {
            tag.set_comment(comment);
        }

        tag.write_to_path(path).context("Failed to save M4A tags")?;
        Ok(())
    }

    fn has_cover_art(&self, path: &Path) -> Result<bool> {
        use mp4ameta::Tag;

        let tag = Tag::read_from_path(path).context("Failed to read M4A tags")?;
        Ok(tag.artworks().count() > 0)
    }

    fn get_cover_art(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        use mp4ameta::Tag;

        let tag = Tag::read_from_path(path).context("Failed to read M4A tags")?;
        if let Some(artwork) = tag.artworks().next() {
            Ok(Some(artwork.data.to_vec()))
        } else {
            Ok(None)
        }
    }

    fn set_cover_art(&self, path: &Path, data: Vec<u8>, mime_type: &str) -> Result<()> {
        use mp4ameta::{Img, Tag};

        let mut tag =
            Tag::read_from_path(path).or_else(|_| Ok::<_, anyhow::Error>(Tag::default()))?;

        // Determine image format from MIME type
        let img = match mime_type {
            "image/jpeg" => Img::jpeg(data),
            "image/png" => Img::png(data),
            _ => anyhow::bail!("Unsupported image format: {}", mime_type),
        };

        tag.set_artwork(img);
        tag.write_to_path(path)
            .context("Failed to save M4A tags with cover art")?;
        Ok(())
    }

    fn remove_cover_art(&self, path: &Path) -> Result<()> {
        use mp4ameta::Tag;

        let mut tag = Tag::read_from_path(path).context("Failed to read M4A tags")?;
        tag.remove_artworks();
        tag.write_to_path(path).context("Failed to save M4A tags")?;
        Ok(())
    }
}

/// Factory function to create the appropriate AudioFile implementation based on file extension
pub fn get_audio_file_handler(extension: &str) -> Option<Box<dyn AudioFile>> {
    match extension.to_lowercase().as_str() {
        "flac" => Some(Box::new(FlacFile)),
        "mp3" => Some(Box::new(Mp3File)),
        "ogg" => Some(Box::new(OggFile)),
        "m4a" => Some(Box::new(M4aFile)),
        _ => None,
    }
}
