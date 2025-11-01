mod library;
mod lyrics;
mod server;

use anyhow::{Context, Result};
use clap::Parser;
use library::MusicLibrary;
use lyrics::LyricDatabase;
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
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with debug level
    // Enable debug logging for symphonia to see audio parsing details
    use tracing_subscriber::EnvFilter;
    
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(tracing::Level::DEBUG.into())
                .add_directive("symphonia=debug".parse().unwrap())
                .add_directive("symphonia_core=debug".parse().unwrap())
                .add_directive("symphonia_format_isomp4=debug".parse().unwrap())
                .add_directive("symphonia_codec_mp3=debug".parse().unwrap())
        )
        .init();

    let cli = Cli::parse();

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

    // Update has_lyrics flags for all tracks
    if let Ok(tracks_with_lyrics) = lyrics_db.get_tracks_with_lyrics().await {
        for track_id in tracks_with_lyrics {
            library.update_track_lyrics_status(&track_id, true).await;
        }
    }

    // Create and start the server
    let app = server::create_router(library, lyrics_db);
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
