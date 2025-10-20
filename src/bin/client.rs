use anyhow::{Context, Result};
use clap::Parser;
use rodio::{Decoder, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::time::Duration;

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
    /// Play a track by ID
    Play { id: String },
    /// Play all tracks in the library
    PlayAll,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::List) {
        Command::List => list_tracks(&cli.server).await?,
        Command::Info { id } => show_track_info(&cli.server, &id).await?,
        Command::Play { id } => play_track(&cli.server, &id).await?,
        Command::PlayAll => play_all_tracks(&cli.server).await?,
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

async fn play_track(server: &str, id: &str) -> Result<()> {
    // Fetch track info first
    let track_url = format!("{}/tracks/{}", server, id);
    let track_response = reqwest::get(&track_url)
        .await
        .context("Failed to connect to server")?;

    if track_response.status() == reqwest::StatusCode::NOT_FOUND {
        anyhow::bail!("Track not found");
    }

    let track: Track = track_response
        .json()
        .await
        .context("Failed to parse track info")?;

    // Display track info
    println!("Now Playing:");
    println!("{:-<80}", "");
    println!("Title:  {}", track.title.as_deref().unwrap_or("Unknown"));
    println!("Artist: {}", track.artist.as_deref().unwrap_or("Unknown"));
    println!("Album:  {}", track.album.as_deref().unwrap_or("Unknown"));
    if let Some(duration) = track.duration_secs {
        let minutes = duration / 60;
        let seconds = duration % 60;
        println!("Duration: {:02}:{:02}", minutes, seconds);
    }
    println!("{:-<80}", "");
    println!();

    // Stream and play the audio
    let stream_url = format!("{}/stream/{}", server, id);
    println!("Streaming from: {}", stream_url);
    
    let response = reqwest::get(&stream_url)
        .await
        .context("Failed to stream audio")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to stream audio: {}", response.status());
    }

    let audio_data = response
        .bytes()
        .await
        .context("Failed to download audio data")?;

    // Create audio output stream
    let (_stream, stream_handle) = OutputStream::try_default()
        .context("Failed to initialize audio output")?;
    
    let sink = Sink::try_new(&stream_handle)
        .context("Failed to create audio sink")?;

    // Decode and play
    let cursor = Cursor::new(audio_data.to_vec());
    let source = Decoder::new(cursor)
        .context("Failed to decode audio")?;
    
    sink.append(source);

    println!("▶️  Playing... (Press Ctrl+C to stop)");
    
    // Wait for playback to finish
    sink.sleep_until_end();

    println!("\n✓ Playback finished");

    Ok(())
}

async fn play_all_tracks(server: &str) -> Result<()> {
    // Get all tracks
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

    println!("Playing {} tracks from the library", tracks.len());
    println!("{:=<80}", "");
    println!();

    // Create audio output stream once for all tracks
    let (_stream, stream_handle) = OutputStream::try_default()
        .context("Failed to initialize audio output")?;
    
    let sink = Sink::try_new(&stream_handle)
        .context("Failed to create audio sink")?;

    // Play each track
    for (idx, track) in tracks.iter().enumerate() {
        println!("[{}/{}] Now Playing:", idx + 1, tracks.len());
        println!("  Title:  {}", track.title.as_deref().unwrap_or("Unknown"));
        println!("  Artist: {}", track.artist.as_deref().unwrap_or("Unknown"));
        println!("  Album:  {}", track.album.as_deref().unwrap_or("Unknown"));
        
        // Stream the audio
        let stream_url = format!("{}/stream/{}", server, track.id);
        
        match reqwest::get(&stream_url).await {
            Ok(response) => {
                if let Ok(audio_data) = response.bytes().await {
                    let cursor = Cursor::new(audio_data.to_vec());
                    if let Ok(source) = Decoder::new(cursor) {
                        sink.append(source);
                        println!("  ▶️  Playing...");
                    } else {
                        println!("  ⚠️  Failed to decode audio");
                    }
                } else {
                    println!("  ⚠️  Failed to download audio");
                }
            }
            Err(e) => {
                println!("  ⚠️  Failed to stream: {}", e);
            }
        }
        
        // Wait a bit for the track to start before queuing next
        tokio::time::sleep(Duration::from_millis(100)).await;
        println!();
    }

    println!("All tracks queued. Playing... (Press Ctrl+C to stop)");
    
    // Wait for all playback to finish
    sink.sleep_until_end();

    println!("\n✓ Playback finished");

    Ok(())
}
