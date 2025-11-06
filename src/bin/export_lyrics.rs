use anyhow::{Context, Result};
use clap::Parser;
use music_station::library::MusicLibrary;
use music_station::lyrics::{LyricDatabase, LyricFormat};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "export-lyrics")]
#[command(about = "Export lyrics from Music Station database to files", long_about = None)]
struct Cli {
    /// Path to music library folder (same as server)
    #[arg(short, long, env = "MUSIC_LIBRARY_PATH")]
    library: PathBuf,

    /// Output directory for exported lyrics files
    #[arg(short, long, default_value = "exported_lyrics")]
    output: PathBuf,
}

/// Sanitize a string to be safe for use in filenames
fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Generate a unique filename with the pattern ARTIST-TITLE-ALBUM.ext
/// Handles duplicate names by adding -1, -2, etc.
fn generate_filename(
    artist: &str,
    title: &str,
    album: &str,
    extension: &str,
    used_names: &mut HashMap<String, usize>,
) -> String {
    let artist_clean = sanitize_filename(artist);
    let title_clean = sanitize_filename(title);
    let album_clean = sanitize_filename(album);

    // Create base name: ARTIST-TITLE-ALBUM
    let base_name = format!("{} - {} - {}", artist_clean, title_clean, album_clean);

    // Check if this name has been used before
    let counter = used_names.entry(base_name.clone()).or_insert(0);

    let filename = if *counter == 0 {
        format!("{}.{}", base_name, extension)
    } else {
        format!("{} - {}.{}", base_name, counter, extension)
    };

    *counter += 1;

    filename
}

/// Get the file extension based on lyric format
fn get_extension(format: &LyricFormat) -> &'static str {
    match format {
        LyricFormat::Plain => "txt",
        LyricFormat::Lrc => "lrc",
        LyricFormat::LrcWord => "wlrc",
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize basic logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let cli = Cli::parse();

    // Validate library path
    if !cli.library.exists() {
        anyhow::bail!("Library path does not exist: {}", cli.library.display());
    }

    if !cli.library.is_dir() {
        anyhow::bail!("Library path is not a directory: {}", cli.library.display());
    }

    tracing::info!("Music Library: {}", cli.library.display());

    // Create output directory if it doesn't exist
    tokio::fs::create_dir_all(&cli.output)
        .await
        .context("Failed to create output directory")?;

    tracing::info!("Output directory: {}", cli.output.display());

    // Initialize music library
    let library = MusicLibrary::new(cli.library.clone());
    library.scan().await.context("Failed to scan library")?;

    // Initialize lyrics database
    let db_path = cli.library.join(".music-station").join("lyrics.db");
    if !db_path.exists() {
        anyhow::bail!(
            "Lyrics database not found at {}. Have you run the server and added lyrics?",
            db_path.display()
        );
    }

    let lyrics_db = LyricDatabase::new(&db_path)
        .await
        .context("Failed to initialize lyrics database")?;

    tracing::info!("Lyrics database: {}", db_path.display());

    // Get all tracks with lyrics
    let track_ids = lyrics_db
        .get_tracks_with_lyrics()
        .await
        .context("Failed to get tracks with lyrics")?;

    if track_ids.is_empty() {
        tracing::warn!("No tracks with lyrics found in database");
        return Ok(());
    }

    tracing::info!("Found {} tracks with lyrics", track_ids.len());

    // Track used filenames to handle duplicates
    let mut used_names: HashMap<String, usize> = HashMap::new();
    let mut exported_count = 0;
    let mut skipped_count = 0;

    // Export each lyric
    for track_id in track_ids {
        // Get track metadata
        let track = match library.get_track(&track_id).await {
            Some(t) => t,
            None => {
                tracing::warn!("Track {} not found in library, skipping", track_id);
                skipped_count += 1;
                continue;
            }
        };

        // Get lyrics
        let lyric = match lyrics_db.get_lyric(&track_id).await {
            Ok(Some(l)) => l,
            Ok(None) => {
                tracing::warn!("Lyrics not found for track {}, skipping", track_id);
                skipped_count += 1;
                continue;
            }
            Err(e) => {
                tracing::error!("Failed to get lyrics for track {}: {}", track_id, e);
                skipped_count += 1;
                continue;
            }
        };

        // Extract metadata with fallbacks
        let artist = track
            .artist
            .as_deref()
            .or(track.album_artist.as_deref())
            .unwrap_or("Unknown Artist");
        let title = track.title.as_deref().unwrap_or("Unknown Title");
        let album = track.album.as_deref().unwrap_or("Unknown Album");

        // Generate filename
        let extension = get_extension(&lyric.format);
        let filename = generate_filename(artist, title, album, extension, &mut used_names);
        let output_path = cli.output.join(&filename);

        // Write lyrics to file
        match tokio::fs::write(&output_path, &lyric.content).await {
            Ok(_) => {
                tracing::info!("Exported: {} -> {}", track_id, filename);
                exported_count += 1;
            }
            Err(e) => {
                tracing::error!("Failed to write {}: {}", filename, e);
                skipped_count += 1;
            }
        }
    }

    tracing::info!("Export complete!");
    tracing::info!("  Exported: {}", exported_count);
    tracing::info!("  Skipped: {}", skipped_count);
    tracing::info!("  Output: {}", cli.output.display());

    Ok(())
}
