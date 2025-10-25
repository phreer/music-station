use music_search_rs::{NetEaseMusicApi, MusicApi, SearchType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== NetEase Cloud Music Search Example ===\n");

    // Create API instance without cookie
    let api: Box<dyn MusicApi> = Box::new(NetEaseMusicApi::new(None)?);

    // Search for songs
    println!("Searching for '告白气球'...");
    let result = api.search("告白气球", SearchType::SongId).await?;
    
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

    // Search for albums
    println!("\n--- Searching for albums ---");
    let album_result = api.search("周杰伦", SearchType::AlbumId).await?;
    
    if album_result.is_success() {
        if let Some(search_result) = album_result.data {
            println!("\nFound {} albums:\n", search_result.album_vos.len());
            
            for (i, album) in search_result.album_vos.iter().enumerate().take(3) {
                println!("{}. {}", i + 1, album.album_name);
                println!("   Artist: {}", album.author_name.join(", "));
                println!("   Songs: {}", album.song_count);
                if let Some(publish_time) = &album.publish_time {
                    println!("   Published: {}", publish_time);
                }
                println!();
            }
        }
    }

    // Get lyrics (example song ID)
    println!("\n--- Getting lyrics ---");
    let lyric_result = api.get_lyric("186016", "186016", false).await?;
    
    if lyric_result.is_success() {
        if let Some(lyric) = lyric_result.data {
            println!("\nLyrics:");
            if let Some(text) = lyric.lyric {
                let lines: Vec<&str> = text.lines().take(10).collect();
                for line in lines {
                    println!("{}", line);
                }
                println!("...(truncated)");
            }
            
            if let Some(trans) = lyric.translate_lyric {
                if !trans.is_empty() {
                    println!("\nTranslation available: {} characters", trans.len());
                }
            }
        }
    }

    Ok(())
}
