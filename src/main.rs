mod library;
mod server;

use anyhow::{Context, Result};
use clap::Parser;
use library::MusicLibrary;
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
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
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
    let library = MusicLibrary::new(cli.library);

    // Scan the library
    library.scan().await.context("Failed to scan library")?;

    // Create and start the server
    let app = server::create_router(library);
    let addr = format!("0.0.0.0:{}", cli.port);

    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("API endpoints:");
    tracing::info!("  GET  /              - API info");
    tracing::info!("  GET  /tracks        - List all tracks");
    tracing::info!("  GET  /tracks/:id    - Get track details");
    tracing::info!("  PUT  /tracks/:id    - Update track metadata");
    tracing::info!("  GET  /stream/:id    - Stream track audio");
    tracing::info!("Web Client:");
    tracing::info!("  http://localhost:{}/web/index.html", cli.port);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("Failed to bind to address")?;

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}
