mod audio;
mod favorites;
mod library;
mod lyrics;
mod playlist;
mod server;
mod stats;

use anyhow::{Context, Result};
use clap::Parser;
use favorites::FavoritesDatabase;
use library::MusicLibrary;
use lyrics::LyricDatabase;
use playlist::PlaylistDatabase;
use stats::StatsDatabase;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "music-station")]
#[command(about = "Music Station Server", long_about = None)]
struct Cli {
    /// Path to music library folder
    #[arg(short, long, env = "MUSIC_LIBRARY_PATH")]
    library: PathBuf,

    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Log level (ignored when RUST_LOG is set)
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing: RUST_LOG takes priority; otherwise use --log-level
    use tracing_subscriber::EnvFilter;

    let env_filter = if std::env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else {
        EnvFilter::new(&cli.log_level)
    };

    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(env_filter)
        .init();

    // Validate library path
    if !cli.library.exists() {
        anyhow::bail!("Library path does not exist: {}", cli.library.display());
    }

    if !cli.library.is_dir() {
        anyhow::bail!("Library path is not a directory: {}", cli.library.display());
    }

    tracing::info!("Starting Music Station");
    tracing::info!("Library path: {}", cli.library.display());

    // Initialize music library
    let library = MusicLibrary::new(cli.library.clone());

    // Scan the library
    library.scan().await.context("Failed to scan library")?;

    // Initialize lyrics database
    let db_path = cli.library.join(".music-station").join("lyrics.db");
    let lyrics_db = LyricDatabase::new(&db_path)
        .await
        .context("Failed to initialize lyrics database")?;

    tracing::info!("Lyrics database: {}", db_path.display());

    // Initialize playlist database
    let playlist_db_path = cli.library.join(".music-station").join("playlists.db");
    let playlist_db = PlaylistDatabase::new(&playlist_db_path)
        .await
        .context("Failed to initialize playlist database")?;

    tracing::info!("Playlist database: {}", playlist_db_path.display());

    // Initialize stats database
    let stats_db_path = cli.library.join(".music-station").join("stats.db");
    let stats_db = StatsDatabase::new(&stats_db_path)
        .await
        .context("Failed to initialize stats database")?;

    tracing::info!("Stats database: {}", stats_db_path.display());

    // Initialize favorites database
    let favorites_db_path = cli.library.join(".music-station").join("favorites.db");
    let favorites_db = FavoritesDatabase::new(&favorites_db_path)
        .await
        .context("Failed to initialize favorites database")?;

    tracing::info!("Favorites database: {}", favorites_db_path.display());

    // Update has_lyrics flags for all tracks
    if let Ok(tracks_with_lyrics) = lyrics_db.get_tracks_with_lyrics().await {
        for track_id in tracks_with_lyrics {
            library.update_track_lyrics_status(&track_id, true).await;
        }
    }

    // Update play counts for all tracks
    if let Ok(play_counts) = stats_db.get_all_play_counts().await {
        for (track_id, count) in play_counts {
            library.update_track_play_count(&track_id, count).await;
        }
    }

    // Warm up the artists cache, then apply favorite flags
    // (get_artists() triggers a cache build from the now-complete track data)
    let _ = library.get_artists().await;
    if let Ok(favorite_names) = favorites_db.get_favorite_artist_names().await {
        for name in favorite_names {
            library.update_artist_favorite_status(&name, true).await;
        }
    }

    // Create and start the server
    let app = server::create_router(library, lyrics_db, playlist_db, stats_db, favorites_db);
    let addr = format!("0.0.0.0:{}", cli.port);

    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("Web Client:");
    tracing::info!("  http://localhost:{}/web/index.html", cli.port);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("Failed to bind to address")?;

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}
