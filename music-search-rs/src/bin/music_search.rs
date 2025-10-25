use music_search_rs::{MusicApi, NetEaseMusicApi, QQMusicApi, SearchSource, SearchType};
use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use tracing_subscriber::{fmt, EnvFilter};

/// Music Search & Lyrics Downloader
/// 
/// Search for songs and download lyrics from NetEase Cloud Music or QQ Music.
#[derive(Parser, Debug)]
#[command(name = "music_search")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Music service to use: 'netease' or 'qq'
    #[arg(short, long, value_name = "SERVICE")]
    api: Option<String>,

    /// Cookie for authentication (can also be set via MUSIC_COOKIE env var)
    #[arg(short, long, env = "MUSIC_COOKIE", value_name = "COOKIE")]
    cookie: Option<String>,

    /// Search query (song name or artist)
    #[arg(short, long, value_name = "QUERY")]
    query: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for logging
    // Use RUST_LOG environment variable to control log level
    // Example: RUST_LOG=debug ./music_search
    // If not set, defaults to showing warnings and errors only
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("warn"));
    
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .init();

    let args = Args::parse();

    println!("=== Music Search & Lyrics Downloader ===\n");

    // Determine API to use
    let api_choice = if let Some(api_name) = args.api {
        match api_name.to_lowercase().as_str() {
            "netease" | "ne" | "163" | "1" => "1".to_string(),
            "qq" | "qqmusic" | "tencent" | "2" => "2".to_string(),
            _ => {
                eprintln!("Invalid API choice: '{}'. Use 'netease' or 'qq'.", api_name);
                return Ok(());
            }
        }
    } else {
        // Interactive selection
        println!("Select music service:");
        println!("1. NetEase Cloud Music");
        println!("2. QQ Music");
        print!("Enter choice (1 or 2): ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        choice.trim().to_string()
    };

    let api: Box<dyn MusicApi> = match api_choice.as_str() {
        "1" => Box::new(NetEaseMusicApi::new(args.cookie.clone())?),
        "2" => Box::new(QQMusicApi::new(args.cookie.clone())?),
        _ => {
            eprintln!("Invalid choice. Exiting.");
            return Ok(());
        }
    };

    let source_name = match api.source() {
        SearchSource::NetEaseMusic => "NetEase Cloud Music",
        SearchSource::QQMusic => "QQ Music",
    };
    println!("\nUsing {} service", source_name);
    
    if args.cookie.is_some() {
        println!("Using provided cookie for authentication");
    }
    println!();

    // Get search query
    let query = if let Some(q) = args.query {
        q
    } else {
        print!("Enter song name or artist to search: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    if query.is_empty() {
        eprintln!("Search query cannot be empty. Exiting.");
        return Ok(());
    }

    // Search for songs
    println!("\nSearching for '{}'...\n", query);
    let search_result = api.search(&query, SearchType::SongId).await?;

    if !search_result.is_success() {
        println!("Search failed: {}", search_result.error_msg.unwrap_or_else(|| "Unknown error".to_string()));
        return Ok(());
    }

    let search_data = search_result.data.as_ref().unwrap();
    
    if search_data.song_vos.is_empty() {
        println!("No results found.");
        return Ok(());
    }

    // Display search results
    println!("Search Results:");
    println!("{:<4} {:<40} {:<30} {:<20}", "No.", "Song", "Artist", "Album");
    println!("{}", "-".repeat(100));

    for (idx, song) in search_data.song_vos.iter().enumerate() {
        let artist_names = song.author_name.join(", ");
        
        // Safely truncate UTF-8 strings by character count
        let song_name = if song.title.chars().count() > 38 {
            let truncated: String = song.title.chars().take(35).collect();
            format!("{}...", truncated)
        } else {
            song.title.clone()
        };

        let artist_display = if artist_names.chars().count() > 28 {
            let truncated: String = artist_names.chars().take(25).collect();
            format!("{}...", truncated)
        } else {
            artist_names
        };

        let album_display = if song.album_name.chars().count() > 18 {
            let truncated: String = song.album_name.chars().take(15).collect();
            format!("{}...", truncated)
        } else {
            song.album_name.clone()
        };

        println!("{:<4} {:<40} {:<30} {:<20}", 
            idx + 1, song_name, artist_display, album_display);
    }

    // Select song
    print!("\nEnter song number to download lyrics (or 0 to exit): ");
    io::stdout().flush()?;
    let mut selection = String::new();
    io::stdin().read_line(&mut selection)?;
    
    let selection: usize = match selection.trim().parse() {
        Ok(n) if n > 0 && n <= search_data.song_vos.len() => n,
        Ok(0) => {
            println!("Exiting.");
            return Ok(());
        }
        _ => {
            eprintln!("Invalid selection. Exiting.");
            return Ok(());
        }
    };

    let selected_song = &search_data.song_vos[selection - 1];
    println!("\nSelected: {} - {}", selected_song.title, 
        selected_song.author_name.join(", "));

    // Get lyrics
    println!("\nFetching lyrics...");
    let lyric_result = api.get_lyric(&selected_song.display_id, &selected_song.display_id, false).await?;

    if !lyric_result.is_success() {
        println!("Failed to get lyrics: {}", lyric_result.error_msg.unwrap_or_else(|| "Unknown error".to_string()));
        return Ok(());
    }

    let lyric_data = lyric_result.data.as_ref().unwrap();

    // Check available lyrics
    let mut available_lyrics = Vec::new();
    if let Some(lyric) = &lyric_data.lyric {
        if !lyric.is_empty() {
            available_lyrics.push(("Original", lyric));
        }
    }
    if let Some(translate) = &lyric_data.translate_lyric {
        if !translate.is_empty() {
            available_lyrics.push(("Translation", translate));
        }
    }
    if let Some(transliteration) = &lyric_data.transliteration_lyric {
        if !transliteration.is_empty() {
            available_lyrics.push(("Transliteration", transliteration));
        }
    }

    if available_lyrics.is_empty() {
        println!("No lyrics found for this song.");
        return Ok(());
    }

    // Display lyrics types available
    println!("\nAvailable lyrics:");
    for (idx, (name, _)) in available_lyrics.iter().enumerate() {
        println!("{}. {}", idx + 1, name);
    }

    // Select lyrics type
    print!("\nEnter lyrics type number to download (or 0 to download all): ");
    io::stdout().flush()?;
    let mut lyric_choice = String::new();
    io::stdin().read_line(&mut lyric_choice)?;
    
    let lyric_choice: usize = match lyric_choice.trim().parse() {
        Ok(n) if n <= available_lyrics.len() => n,
        _ => {
            eprintln!("Invalid selection. Exiting.");
            return Ok(());
        }
    };

    // Create output directory
    let output_dir = "lyrics";
    fs::create_dir_all(output_dir)?;

    // Sanitize filename
    let safe_filename = sanitize_filename(&format!(
        "{} - {}",
        selected_song.title,
        selected_song.author_name.join(", ")
    ));

    // Download and save lyrics
    if lyric_choice == 0 {
        // Download all lyrics types
        for (name, content) in &available_lyrics {
            let filename = format!("{}_{}.lrc", safe_filename, name.to_lowercase());
            let filepath = Path::new(output_dir).join(&filename);
            
            fs::write(&filepath, content)?;
            println!("✓ Saved {} lyrics to: {}", name, filepath.display());
        }
    } else {
        // Download selected lyrics type
        let (name, content) = &available_lyrics[lyric_choice - 1];
        let filename = format!("{}_{}.lrc", safe_filename, name.to_lowercase());
        let filepath = Path::new(output_dir).join(&filename);
        
        fs::write(&filepath, content)?;
        println!("✓ Saved {} lyrics to: {}", name, filepath.display());
    }

    println!("\n✓ Download complete!");
    Ok(())
}

/// Sanitize filename by removing or replacing invalid characters
fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}
