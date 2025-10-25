use music_search_rs::{QQMusicApi, MusicApi, SearchType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== QQ Music Search Example ===\n");

    // Create API instance without cookie
    let api: Box<dyn MusicApi> = Box::new(QQMusicApi::new(None)?);

    // Search for songs
    println!("Searching for '告白气球'...");
    let result = match api.search("告白气球", SearchType::SongId).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("failed to search result: {}", e);
            return Err(Box::<dyn std::error::Error>::from(e));
        }
    };
    
    if result.is_success() {
        if let Some(search_result) = result.data {
            println!("\nFound {} songs:\n", search_result.song_vos.len());
            
            for (i, song) in search_result.song_vos.iter().enumerate().take(5) {
                println!("{}. {} - {}", 
                    i + 1,
                    song.title, 
                    song.author_name.join(", ")
                );
                println!("   Album: {}", song.album_name);
                println!("   Duration: {}s\n", song.duration / 1000);
            }
        }
    } else {
        println!("Search failed: {:?}", result.error_msg);
    }

    // Search for playlists
    println!("\n--- Searching for playlists ---");
    let playlist_result = api.search("周杰伦", SearchType::PlaylistId).await?;
    
    if playlist_result.is_success() {
        if let Some(search_result) = playlist_result.data {
            println!("\nFound {} playlists:\n", search_result.playlist_vos.len());
            
            for (i, playlist) in search_result.playlist_vos.iter().enumerate().take(3) {
                println!("{}. {}", i + 1, playlist.playlist_name);
                println!("   Creator: {}", playlist.author_name);
                println!("   Songs: {}", playlist.song_count);
                println!("   Plays: {}", playlist.play_count);
                if let Some(desc) = &playlist.description {
                    // Properly truncate UTF-8 string by character count
                    let short_desc = if desc.chars().count() > 50 {
                        let truncated: String = desc.chars().take(50).collect();
                        format!("{}...", truncated)
                    } else {
                        desc.clone()
                    };
                    println!("   Description: {}", short_desc);
                }
                println!();
            }
        }
    }

    // Get a song's details
    println!("\n--- Getting song details ---");
    let songs_result = api.get_songs(&["001RaE0n4RrGX9".to_string()]).await?;
    
    for (id, song_vo) in songs_result.iter() {
        if song_vo.is_success() {
            if let Some(song) = &song_vo.data {
                println!("\nSong Details:");
                println!("  ID: {}", song.id);
                println!("  Name: {}", song.name);
                println!("  Singer: {}", song.singer.join(", "));
                println!("  Album: {}", song.album);
                println!("  Duration: {}s", song.duration / 1000);
            }
        } else {
            println!("Failed to get song {}: {:?}", id, song_vo.error_msg);
        }
    }

    Ok(())
}
