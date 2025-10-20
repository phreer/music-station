use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Track {
    id: String,
    path: std::path::PathBuf,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    duration_secs: Option<u64>,
    file_size: u64,
}

#[derive(Parser)]
#[command(name = "music-client")]
#[command(about = "Music Station CLI Client", long_about = None)]
struct Cli {
    /// Server URL
    #[arg(short, long, default_value = "http://localhost:3000")]
    server: String,

    /// Command to execute
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Parser)]
enum Command {
    /// List all tracks
    List,
    /// Show track details
    Info { id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::List) {
        Command::List => list_tracks(&cli.server).await?,
        Command::Info { id } => show_track_info(&cli.server, &id).await?,
    }

    Ok(())
}

async fn list_tracks(server: &str) -> Result<()> {
    let url = format!("{}/tracks", server);
    let response = reqwest::get(&url)
        .await
        .context("Failed to connect to server")?;

    if !response.status().is_success() {
        anyhow::bail!("Server returned error: {}", response.status());
    }

    let tracks: Vec<Track> = response.json().await.context("Failed to parse response")?;

    if tracks.is_empty() {
        println!("No tracks found in the library.");
        return Ok(());
    }

    println!("Music Library ({} tracks):", tracks.len());
    println!("{:-<80}", "");
    
    for (idx, track) in tracks.iter().enumerate() {
        let title = track.title.as_deref().unwrap_or("Unknown Title");
        let artist = track.artist.as_deref().unwrap_or("Unknown Artist");
        let album = track.album.as_deref().unwrap_or("Unknown Album");
        
        println!("{}. {} - {}", idx + 1, artist, title);
        println!("   Album: {}", album);
        
        if let Some(duration) = track.duration_secs {
            let minutes = duration / 60;
            let seconds = duration % 60;
            println!("   Duration: {:02}:{:02}", minutes, seconds);
        }
        
        println!("   File: {}", track.path.display());
        println!("   ID: {}", track.id);
        println!("   Stream: {}/stream/{}", server, track.id);
        println!();
    }

    Ok(())
}

async fn show_track_info(server: &str, id: &str) -> Result<()> {
    let url = format!("{}/tracks/{}", server, id);
    let response = reqwest::get(&url)
        .await
        .context("Failed to connect to server")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        anyhow::bail!("Track not found");
    }

    if !response.status().is_success() {
        anyhow::bail!("Server returned error: {}", response.status());
    }

    let track: Track = response.json().await.context("Failed to parse response")?;

    println!("Track Information:");
    println!("{:-<80}", "");
    println!("Title:    {}", track.title.as_deref().unwrap_or("Unknown"));
    println!("Artist:   {}", track.artist.as_deref().unwrap_or("Unknown"));
    println!("Album:    {}", track.album.as_deref().unwrap_or("Unknown"));
    
    if let Some(duration) = track.duration_secs {
        let minutes = duration / 60;
        let seconds = duration % 60;
        println!("Duration: {:02}:{:02}", minutes, seconds);
    }
    
    println!("File:     {}", track.path.display());
    println!("Size:     {} bytes", track.file_size);
    println!("ID:       {}", track.id);
    println!("Stream:   {}/stream/{}", server, track.id);

    Ok(())
}
