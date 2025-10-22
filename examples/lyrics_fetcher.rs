//! Example demonstrating the lyrics fetching API
//! 
//! Run this example with:
//! ```
//! cargo run --example lyrics_fetcher
//! ```

use music_station::lyrics::fetcher::*;
use music_station::lyrics::providers::*;
use music_station::lyrics::LyricFormat;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("=== Lyrics Fetching API Example ===\n");

    // Example 1: Using a single provider
    println!("Example 1: Single Provider\n");
    {
        let provider = MockLyricsProvider::new("example-provider")
            .add_lyrics(
                "Bohemian Rhapsody",
                "Queen",
                "[00:00.00]Is this the real life?\n[00:05.00]Is this just fantasy?".to_string(),
                LyricFormat::Lrc,
            )
            .add_lyrics(
                "Imagine",
                "John Lennon",
                "Imagine there's no heaven\nIt's easy if you try\nNo hell below us\nAbove us only sky".to_string(),
                LyricFormat::Plain,
            );

        let query = LyricsQuery::new("Bohemian Rhapsody")
            .with_artist("Queen");

        println!("Searching for: {} by {}", query.title, query.artist.as_ref().unwrap());
        
        match provider.search_and_fetch(&query).await? {
            Some(lyrics) => {
                println!("✓ Found lyrics!");
                println!("  Source: {}", lyrics.source);
                println!("  Format: {:?}", lyrics.format);
                println!("  Content preview: {}", lyrics.content.lines().take(2).collect::<Vec<_>>().join("\n"));
            }
            None => println!("✗ No lyrics found"),
        }
    }

    println!("\n---\n");

    // Example 2: Using the aggregator with multiple providers
    println!("Example 2: Multiple Providers with Fallback\n");
    {
        let provider1 = MockLyricsProvider::new("provider-1")
            .add_lyrics(
                "Hotel California",
                "Eagles",
                "[00:00.00]On a dark desert highway...".to_string(),
                LyricFormat::Lrc,
            );

        let provider2 = MockLyricsProvider::new("provider-2")
            .add_lyrics(
                "Stairway to Heaven",
                "Led Zeppelin",
                "There's a lady who's sure\nAll that glitters is gold".to_string(),
                LyricFormat::Plain,
            );

        let aggregator = LyricsAggregator::new()
            .add_provider(Box::new(provider1))
            .add_provider(Box::new(provider2));

        println!("Registered providers: {:?}", aggregator.provider_names());

        // Try to find lyrics that only exist in provider-2
        let query = LyricsQuery::new("Stairway to Heaven")
            .with_artist("Led Zeppelin");

        println!("\nSearching for: {} by {}", query.title, query.artist.as_ref().unwrap());

        match aggregator.fetch_lyrics(&query).await? {
            Some(lyrics) => {
                println!("✓ Found lyrics from: {}", lyrics.source);
                println!("  Format: {:?}", lyrics.format);
                println!("  Language: {:?}", lyrics.language);
            }
            None => println!("✗ No lyrics found from any provider"),
        }
    }

    println!("\n---\n");

    // Example 3: Search all providers
    println!("Example 3: Search All Providers\n");
    {
        let provider1 = MockLyricsProvider::new("genius")
            .add_lyrics(
                "Yesterday",
                "The Beatles",
                "Yesterday, all my troubles seemed so far away".to_string(),
                LyricFormat::Plain,
            );

        let provider2 = MockLyricsProvider::new("netease")
            .add_lyrics(
                "Yesterday",
                "The Beatles",
                "[00:00.00]Yesterday, all my troubles seemed so far away".to_string(),
                LyricFormat::Lrc,
            );

        let aggregator = LyricsAggregator::new()
            .add_provider(Box::new(provider1))
            .add_provider(Box::new(provider2));

        let query = LyricsQuery::new("Yesterday").with_artist("The Beatles");
        println!("Searching all providers for: {} by {}", query.title, query.artist.as_ref().unwrap());

        let results = aggregator.search_all(&query).await;

        for (provider_name, result) in results {
            match result {
                Ok(matches) => {
                    println!("\n  Provider '{}': {} results", provider_name, matches.len());
                    for m in matches {
                        println!("    - {} by {} (confidence: {:.2})", m.title, m.artist, m.confidence);
                    }
                }
                Err(e) => {
                    println!("\n  Provider '{}': Error - {:?}", provider_name, e);
                }
            }
        }
    }

    println!("\n---\n");

    // Example 4: Health checks
    println!("Example 4: Provider Health Checks\n");
    {
        let provider1 = MockLyricsProvider::new("healthy-provider")
            .add_lyrics("Test", "Test", "Test".to_string(), LyricFormat::Plain);

        let provider2 = MockLyricsProvider::new("empty-provider");

        let aggregator = LyricsAggregator::new()
            .add_provider(Box::new(provider1))
            .add_provider(Box::new(provider2));

        println!("Checking provider health...");
        let health = aggregator.health_check_all().await;

        for (provider_name, is_healthy) in health {
            println!("  {}: {}", provider_name, if is_healthy { "✓ Healthy" } else { "✗ Unhealthy" });
        }
    }

    println!("\n=== End of Examples ===");

    Ok(())
}
