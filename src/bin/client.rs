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
    play_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Playlist {
    id: String,
    name: String,
    description: Option<String>,
    tracks: Vec<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct PlaylistCreate {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct PlaylistUpdate {
    name: Option<String>,
    description: Option<String>,
    tracks: Option<Vec<String>>,
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
    /// Playlist management commands
    #[command(subcommand)]
    Playlist(PlaylistCommand),
}

#[derive(Parser)]
enum PlaylistCommand {
    /// List all playlists
    List,
    /// Create a new playlist
    Create {
        /// Playlist name
        name: String,
        /// Playlist description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Show playlist details
    Info {
        /// Playlist ID
        id: String,
    },
    /// Update playlist
    Update {
        /// Playlist ID
        id: String,
        /// New name
        #[arg(short, long)]
        name: Option<String>,
        /// New description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Delete a playlist
    Delete {
        /// Playlist ID
        id: String,
    },
    /// Add track to playlist
    AddTrack {
        /// Playlist ID
        playlist_id: String,
        /// Track IDs to add
        track_ids: Vec<String>,
    },
    /// Remove track from playlist
    RemoveTrack {
        /// Playlist ID
        playlist_id: String,
        /// Track IDs to remove
        track_ids: Vec<String>,
    },
    /// Play all tracks in a playlist
    Play {
        /// Playlist ID
        id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::List) {
        Command::List => list_tracks(&cli.server).await?,
        Command::Info { id } => show_track_info(&cli.server, &id).await?,
        Command::Play { id } => play_track(&cli.server, &id).await?,
        Command::PlayAll => play_all_tracks(&cli.server).await?,
        Command::Playlist(playlist_cmd) => {
            handle_playlist_command(&cli.server, playlist_cmd).await?
        }
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
    println!("Plays:    {}", track.play_count);
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

    // Increment play count
    let play_url = format!("{}/tracks/{}/play", server, id);
    let client = reqwest::Client::new();
    if let Err(e) = client.post(&play_url).send().await {
        eprintln!("Warning: Failed to increment play count: {}", e);
    }

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
    let (_stream, stream_handle) =
        OutputStream::try_default().context("Failed to initialize audio output")?;

    let sink = Sink::try_new(&stream_handle).context("Failed to create audio sink")?;

    // Decode and play
    let cursor = Cursor::new(audio_data.to_vec());
    let source = Decoder::new(cursor).context("Failed to decode audio")?;

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
    let (_stream, stream_handle) =
        OutputStream::try_default().context("Failed to initialize audio output")?;

    let sink = Sink::try_new(&stream_handle).context("Failed to create audio sink")?;

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

async fn handle_playlist_command(server: &str, cmd: PlaylistCommand) -> Result<()> {
    match cmd {
        PlaylistCommand::List => list_playlists(server).await,
        PlaylistCommand::Create { name, description } => {
            create_playlist(server, name, description).await
        }
        PlaylistCommand::Info { id } => show_playlist_info(server, &id).await,
        PlaylistCommand::Update {
            id,
            name,
            description,
        } => update_playlist(server, &id, name, description).await,
        PlaylistCommand::Delete { id } => delete_playlist(server, &id).await,
        PlaylistCommand::AddTrack {
            playlist_id,
            track_ids,
        } => add_tracks_to_playlist(server, &playlist_id, track_ids).await,
        PlaylistCommand::RemoveTrack {
            playlist_id,
            track_ids,
        } => remove_tracks_from_playlist(server, &playlist_id, track_ids).await,
        PlaylistCommand::Play { id } => play_playlist(server, &id).await,
    }
}

async fn list_playlists(server: &str) -> Result<()> {
    let url = format!("{}/playlists", server);
    let response = reqwest::get(&url)
        .await
        .context("Failed to connect to server")?;

    if !response.status().is_success() {
        anyhow::bail!("Server returned error: {}", response.status());
    }

    let playlists: Vec<Playlist> = response.json().await.context("Failed to parse response")?;

    if playlists.is_empty() {
        println!("No playlists found.");
        return Ok(());
    }

    println!("Playlists ({}):", playlists.len());
    println!("{:-<80}", "");

    for playlist in playlists.iter() {
        println!("📋 {}", playlist.name);
        if let Some(desc) = &playlist.description {
            println!("   Description: {}", desc);
        }
        println!("   Tracks: {}", playlist.tracks.len());
        println!("   ID: {}", playlist.id);
        println!("   Created: {}", playlist.created_at);
        println!("   Updated: {}", playlist.updated_at);
        println!();
    }

    Ok(())
}

async fn create_playlist(server: &str, name: String, description: Option<String>) -> Result<()> {
    let url = format!("{}/playlists", server);
    let client = reqwest::Client::new();

    let body = PlaylistCreate { name, description };

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("Failed to connect to server")?;

    if !response.status().is_success() {
        anyhow::bail!("Server returned error: {}", response.status());
    }

    let playlist: Playlist = response.json().await.context("Failed to parse response")?;

    println!("✓ Playlist created successfully!");
    println!("ID: {}", playlist.id);
    println!("Name: {}", playlist.name);
    if let Some(desc) = &playlist.description {
        println!("Description: {}", desc);
    }

    Ok(())
}

async fn show_playlist_info(server: &str, id: &str) -> Result<()> {
    let url = format!("{}/playlists/{}", server, id);
    let response = reqwest::get(&url)
        .await
        .context("Failed to connect to server")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        anyhow::bail!("Playlist not found");
    }

    if !response.status().is_success() {
        anyhow::bail!("Server returned error: {}", response.status());
    }

    let playlist: Playlist = response.json().await.context("Failed to parse response")?;

    println!("Playlist Information:");
    println!("{:-<80}", "");
    println!("Name:        {}", playlist.name);
    if let Some(desc) = &playlist.description {
        println!("Description: {}", desc);
    }
    println!("ID:          {}", playlist.id);
    println!("Created:     {}", playlist.created_at);
    println!("Updated:     {}", playlist.updated_at);
    println!("Tracks:      {}", playlist.tracks.len());

    if !playlist.tracks.is_empty() {
        println!("\nTracks:");
        for (idx, track_id) in playlist.tracks.iter().enumerate() {
            // Try to fetch track info
            let track_url = format!("{}/tracks/{}", server, track_id);
            match reqwest::get(&track_url).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(track) = resp.json::<Track>().await {
                        let title = track.title.as_deref().unwrap_or("Unknown");
                        let artist = track.artist.as_deref().unwrap_or("Unknown");
                        println!("  {}. {} - {}", idx + 1, artist, title);
                    } else {
                        println!("  {}. {} (details unavailable)", idx + 1, track_id);
                    }
                }
                _ => println!("  {}. {} (not found)", idx + 1, track_id),
            }
        }
    }

    Ok(())
}

async fn update_playlist(
    server: &str,
    id: &str,
    name: Option<String>,
    description: Option<String>,
) -> Result<()> {
    if name.is_none() && description.is_none() {
        anyhow::bail!("At least one of --name or --description must be provided");
    }

    let url = format!("{}/playlists/{}", server, id);
    let client = reqwest::Client::new();

    let body = PlaylistUpdate {
        name,
        description,
        tracks: None,
    };

    let response = client
        .put(&url)
        .json(&body)
        .send()
        .await
        .context("Failed to connect to server")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        anyhow::bail!("Playlist not found");
    }

    if !response.status().is_success() {
        anyhow::bail!("Server returned error: {}", response.status());
    }

    let playlist: Playlist = response.json().await.context("Failed to parse response")?;

    println!("✓ Playlist updated successfully!");
    println!("Name: {}", playlist.name);
    if let Some(desc) = &playlist.description {
        println!("Description: {}", desc);
    }

    Ok(())
}

async fn delete_playlist(server: &str, id: &str) -> Result<()> {
    let url = format!("{}/playlists/{}", server, id);
    let client = reqwest::Client::new();

    let response = client
        .delete(&url)
        .send()
        .await
        .context("Failed to connect to server")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        anyhow::bail!("Playlist not found");
    }

    if !response.status().is_success() {
        anyhow::bail!("Server returned error: {}", response.status());
    }

    println!("✓ Playlist deleted successfully!");

    Ok(())
}

async fn add_tracks_to_playlist(
    server: &str,
    playlist_id: &str,
    track_ids: Vec<String>,
) -> Result<()> {
    // Get current playlist
    let url = format!("{}/playlists/{}", server, playlist_id);
    let response = reqwest::get(&url)
        .await
        .context("Failed to connect to server")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        anyhow::bail!("Playlist not found");
    }

    let mut playlist: Playlist = response.json().await.context("Failed to parse response")?;

    // Add new tracks (avoid duplicates)
    let mut added = 0;
    for track_id in track_ids {
        if !playlist.tracks.contains(&track_id) {
            playlist.tracks.push(track_id);
            added += 1;
        }
    }

    if added == 0 {
        println!("No new tracks added (all tracks already in playlist)");
        return Ok(());
    }

    // Update playlist
    let client = reqwest::Client::new();
    let body = PlaylistUpdate {
        name: None,
        description: None,
        tracks: Some(playlist.tracks),
    };

    let response = client
        .put(&url)
        .json(&body)
        .send()
        .await
        .context("Failed to update playlist")?;

    if !response.status().is_success() {
        anyhow::bail!("Server returned error: {}", response.status());
    }

    println!("✓ Added {} track(s) to playlist!", added);

    Ok(())
}

async fn remove_tracks_from_playlist(
    server: &str,
    playlist_id: &str,
    track_ids: Vec<String>,
) -> Result<()> {
    // Get current playlist
    let url = format!("{}/playlists/{}", server, playlist_id);
    let response = reqwest::get(&url)
        .await
        .context("Failed to connect to server")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        anyhow::bail!("Playlist not found");
    }

    let mut playlist: Playlist = response.json().await.context("Failed to parse response")?;

    // Remove tracks
    let original_count = playlist.tracks.len();
    playlist.tracks.retain(|id| !track_ids.contains(id));
    let removed = original_count - playlist.tracks.len();

    if removed == 0 {
        println!("No tracks removed (tracks not found in playlist)");
        return Ok(());
    }

    // Update playlist
    let client = reqwest::Client::new();
    let body = PlaylistUpdate {
        name: None,
        description: None,
        tracks: Some(playlist.tracks),
    };

    let response = client
        .put(&url)
        .json(&body)
        .send()
        .await
        .context("Failed to update playlist")?;

    if !response.status().is_success() {
        anyhow::bail!("Server returned error: {}", response.status());
    }

    println!("✓ Removed {} track(s) from playlist!", removed);

    Ok(())
}

async fn play_playlist(server: &str, id: &str) -> Result<()> {
    // Get playlist
    let url = format!("{}/playlists/{}", server, id);
    let response = reqwest::get(&url)
        .await
        .context("Failed to connect to server")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        anyhow::bail!("Playlist not found");
    }

    let playlist: Playlist = response.json().await.context("Failed to parse response")?;

    if playlist.tracks.is_empty() {
        println!("Playlist is empty");
        return Ok(());
    }

    println!("Playing playlist: {}", playlist.name);
    if let Some(desc) = &playlist.description {
        println!("Description: {}", desc);
    }
    println!("Tracks: {}", playlist.tracks.len());
    println!("{:=<80}", "");
    println!();

    // Create audio output stream once for all tracks
    let (_stream, stream_handle) =
        OutputStream::try_default().context("Failed to initialize audio output")?;

    let sink = Sink::try_new(&stream_handle).context("Failed to create audio sink")?;

    // Play each track
    for (idx, track_id) in playlist.tracks.iter().enumerate() {
        // Fetch track info
        let track_url = format!("{}/tracks/{}", server, track_id);
        match reqwest::get(&track_url).await {
            Ok(track_response) if track_response.status().is_success() => {
                if let Ok(track) = track_response.json::<Track>().await {
                    println!("[{}/{}] Now Playing:", idx + 1, playlist.tracks.len());
                    println!("  Title:  {}", track.title.as_deref().unwrap_or("Unknown"));
                    println!("  Artist: {}", track.artist.as_deref().unwrap_or("Unknown"));
                    println!("  Album:  {}", track.album.as_deref().unwrap_or("Unknown"));

                    // Stream the audio
                    let stream_url = format!("{}/stream/{}", server, track_id);

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
                } else {
                    println!(
                        "[{}/{}] ⚠️  Track {} (invalid metadata)",
                        idx + 1,
                        playlist.tracks.len(),
                        track_id
                    );
                }
            }
            _ => {
                println!(
                    "[{}/{}] ⚠️  Track {} not found",
                    idx + 1,
                    playlist.tracks.len(),
                    track_id
                );
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
