use anyhow::{Context, Result};
use clap::Parser;
use music_station::library::MusicLibrary;
use music_station::lyrics::LyricDatabase;
use music_station::playlist::PlaylistDatabase;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "migrate-track-ids")]
#[command(about = "Migrate track IDs from absolute paths to relative paths", long_about = None)]
struct Cli {
    /// Path to music library folder
    #[arg(short, long, env = "MUSIC_LIBRARY_PATH")]
    library: PathBuf,

    /// Perform a dry run without making changes
    #[arg(long, default_value = "false")]
    dry_run: bool,
}

/// Generate the old track ID (based on absolute path)
fn generate_old_id(absolute_path: &str) -> String {
    format!("{:x}", md5::compute(absolute_path.as_bytes()))
}

/// Generate the new track ID (based on relative path)
fn generate_new_id(relative_path: &str) -> String {
    format!("{:x}", md5::compute(relative_path.as_bytes()))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
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

    if cli.dry_run {
        tracing::info!("=== DRY RUN MODE - No changes will be made ===");
    }

    tracing::info!("Music Library: {}", cli.library.display());

    // Initialize music library to scan all tracks
    let library = MusicLibrary::new(cli.library.clone());
    library.scan().await.context("Failed to scan library")?;

    let tracks = library.get_tracks().await;
    tracing::info!("Found {} tracks in library", tracks.len());

    // Build mapping of old IDs to new IDs
    let mut id_mapping: HashMap<String, String> = HashMap::new();

    for track in &tracks {
        let absolute_path = track.path.to_string_lossy().to_string();
        let relative_path = track
            .path
            .strip_prefix(&cli.library)
            .unwrap_or(&track.path)
            .to_string_lossy()
            .to_string();

        let old_id = generate_old_id(&absolute_path);
        let new_id = generate_new_id(&relative_path);

        if old_id != new_id {
            id_mapping.insert(old_id, new_id);
        }
    }

    tracing::info!("Found {} tracks that need ID migration", id_mapping.len());

    if id_mapping.is_empty() {
        tracing::info!("No migration needed - all track IDs are already using relative paths");
        return Ok(());
    }

    // Migrate lyrics database
    let db_path = cli.library.join(".music-station").join("lyrics.db");
    if db_path.exists() {
        tracing::info!("Migrating lyrics database: {}", db_path.display());

        let lyrics_db = LyricDatabase::new(&db_path)
            .await
            .context("Failed to open lyrics database")?;

        let track_ids_with_lyrics = lyrics_db
            .get_tracks_with_lyrics()
            .await
            .context("Failed to get tracks with lyrics")?;

        let mut migrated_count = 0;
        let mut skipped_count = 0;

        for old_track_id in track_ids_with_lyrics {
            if let Some(new_track_id) = id_mapping.get(&old_track_id) {
                // Get the lyric with old ID
                if let Some(lyric) = lyrics_db.get_lyric(&old_track_id).await? {
                    tracing::info!("  Migrating lyric: {} -> {}", old_track_id, new_track_id);

                    if !cli.dry_run {
                        // Save lyric with new ID
                        lyrics_db
                            .save_lyric(
                                new_track_id,
                                lyric.content,
                                lyric.format,
                                lyric.language,
                                lyric.source,
                            )
                            .await
                            .context(format!(
                                "Failed to save lyric with new ID: {}",
                                new_track_id
                            ))?;

                        // Delete old lyric entry
                        lyrics_db
                            .delete_lyric(&old_track_id)
                            .await
                            .context(format!("Failed to delete old lyric: {}", old_track_id))?;
                    }

                    migrated_count += 1;
                } else {
                    tracing::warn!("  Lyric not found for track ID: {}", old_track_id);
                    skipped_count += 1;
                }
            } else {
                // Track ID doesn't need migration (already using relative path)
                skipped_count += 1;
            }
        }

        tracing::info!("Lyrics Database Migration:");
        tracing::info!("  Migrated: {}", migrated_count);
        tracing::info!("  Skipped: {}", skipped_count);
    } else {
        tracing::info!("No lyrics database found at {}", db_path.display());
    }

    // Migrate playlist database
    let playlist_db_path = cli.library.join(".music-station").join("playlists.db");
    if playlist_db_path.exists() {
        tracing::info!(
            "Migrating playlist database: {}",
            playlist_db_path.display()
        );

        let playlist_db = PlaylistDatabase::new(&playlist_db_path)
            .await
            .context("Failed to open playlist database")?;

        let playlists = playlist_db
            .get_playlists()
            .await
            .context("Failed to get playlists")?;

        let mut migrated_playlists = 0;
        let mut migrated_tracks = 0;

        for playlist in playlists {
            let mut has_changes = false;
            let mut new_track_ids = Vec::new();

            for old_track_id in &playlist.tracks {
                if let Some(new_track_id) = id_mapping.get(old_track_id) {
                    tracing::info!(
                        "  Migrating track in playlist '{}': {} -> {}",
                        playlist.name,
                        &old_track_id[..8],
                        &new_track_id[..8]
                    );
                    new_track_ids.push(new_track_id.clone());
                    has_changes = true;
                    migrated_tracks += 1;
                } else {
                    new_track_ids.push(old_track_id.clone());
                }
            }

            if has_changes && !cli.dry_run {
                // Update playlist with new track IDs
                use music_station::playlist::PlaylistUpdate;
                playlist_db
                    .update_playlist(
                        &playlist.id,
                        PlaylistUpdate {
                            name: None,
                            description: None,
                            tracks: Some(new_track_ids),
                        },
                    )
                    .await
                    .context(format!("Failed to update playlist: {}", playlist.id))?;

                migrated_playlists += 1;
            } else if has_changes {
                migrated_playlists += 1;
            }
        }

        tracing::info!("Playlist Database Migration:");
        tracing::info!("  Playlists updated: {}", migrated_playlists);
        tracing::info!("  Tracks migrated: {}", migrated_tracks);
    } else {
        tracing::info!(
            "No playlist database found at {}",
            playlist_db_path.display()
        );
    }

    if cli.dry_run {
        tracing::info!("");
        tracing::info!("=== DRY RUN COMPLETE - No changes were made ===");
        tracing::info!("Run without --dry-run to perform the migration");
    } else {
        tracing::info!("");
        tracing::info!("=== MIGRATION COMPLETE ===");
        tracing::info!("Track IDs have been updated to use relative paths");
        tracing::info!("You should restart the Music Station server to pick up the changes");
    }

    Ok(())
}
