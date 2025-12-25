use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::library::{Album, Artist, LibraryStats, MusicLibrary, Track, TrackMetadataUpdate};
use crate::lyrics::fetcher::LyricsProvider as LyricsProviderTrait;
use crate::lyrics::fetcher::{
    LyricsQuery, LyricsResponse, LyricsSearchResult as FetcherSearchResult,
};
use crate::lyrics::music_search_provider::{NetEaseLyricsProvider, QQMusicLyricsProvider};
use crate::lyrics::{Lyric, LyricDatabase, LyricFormat, LyricUpload};
use crate::playlist::{Playlist, PlaylistCreate, PlaylistDatabase, PlaylistUpdate};
use crate::stats::StatsDatabase;

#[derive(Clone)]
pub struct AppState {
    pub library: MusicLibrary,
    pub lyrics_db: LyricDatabase,
    pub playlist_db: PlaylistDatabase,
    pub stats_db: StatsDatabase,
    pub netease_provider: Option<std::sync::Arc<NetEaseLyricsProvider>>,
    pub qqmusic_provider: Option<std::sync::Arc<QQMusicLyricsProvider>>,
}

pub fn create_router(
    library: MusicLibrary,
    lyrics_db: LyricDatabase,
    playlist_db: PlaylistDatabase,
    stats_db: StatsDatabase,
) -> Router {
    // Initialize lyrics providers
    let netease_provider = NetEaseLyricsProvider::new(None)
        .map(|p| std::sync::Arc::new(p))
        .ok();
    let qqmusic_provider = QQMusicLyricsProvider::new(None)
        .map(|p| std::sync::Arc::new(p))
        .ok();

    if netease_provider.is_none() {
        tracing::warn!("Failed to initialize NetEase lyrics provider");
    }
    if qqmusic_provider.is_none() {
        tracing::warn!("Failed to initialize QQ Music lyrics provider");
    }

    let state = AppState {
        library,
        lyrics_db,
        playlist_db,
        stats_db,
        netease_provider,
        qqmusic_provider,
    };

    // Serve static files from ./static directory
    let static_service = ServeDir::new("static");

    Router::new()
        .route("/", get(root))
        .route("/tracks", get(list_tracks))
        .route("/tracks/:id", get(get_track).put(update_track))
        .route(
            "/tracks/:id/play",
            axum::routing::post(increment_play_count),
        )
        .route("/stream/:id", get(stream_track))
        .route(
            "/cover/:id",
            get(get_cover).post(upload_cover).delete(delete_cover),
        )
        .route(
            "/lyrics/:id",
            get(get_lyrics).put(upload_lyrics).delete(delete_lyrics),
        )
        .route("/lyrics/search", get(search_lyrics))
        .route(
            "/lyrics/fetch/:provider/:song_id",
            get(fetch_lyrics_from_provider),
        )
        .route("/albums", get(list_albums))
        .route("/albums/:name", get(get_album))
        .route("/artists", get(list_artists))
        .route("/artists/:name", get(get_artist))
        .route("/stats", get(get_stats))
        .route("/playlists", get(list_playlists).post(create_playlist))
        .route(
            "/playlists/:id",
            get(get_playlist)
                .put(update_playlist)
                .delete(delete_playlist),
        )
        .route(
            "/playlists/:id/tracks/:track_id",
            axum::routing::post(add_track_to_playlist).delete(remove_track_from_playlist),
        )
        .nest_service("/web", static_service)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Root endpoint
async fn root() -> &'static str {
    "Music Station API v0.1.0"
}

/// List all tracks
async fn list_tracks(State(state): State<AppState>) -> Json<Vec<Track>> {
    tracing::debug!("Fetching all tracks");
    let tracks = state.library.get_tracks().await;
    tracing::debug!("Returning {} tracks", tracks.len());
    Json(tracks)
}

/// Get a specific track by ID
async fn get_track(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Track>, StatusCode> {
    tracing::debug!("Fetching track with id: {}", id);
    let result = state
        .library
        .get_track(&id)
        .await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND);

    if result.is_ok() {
        tracing::debug!("Track {} found", id);
    } else {
        tracing::warn!("Track {} not found", id);
    }

    result
}

/// Increment play count for a track
async fn increment_play_count(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<u64>, StatusCode> {
    tracing::debug!("Incrementing play count for track: {}", id);

    // Check if track exists
    if state.library.get_track(&id).await.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let count = state
        .stats_db
        .increment_play_count(&id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to increment play count: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Update in-memory library
    state.library.update_track_play_count(&id, count).await;

    Ok(Json(count))
}

/// Stream a track by ID with HTTP Range support
async fn stream_track(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    tracing::debug!("Streaming track with id: {}", id);
    let track = state
        .library
        .get_track(&id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    tracing::debug!("Streaming file: {}", track.path.display());

    // Determine content type based on file extension
    let content_type = match track.path.extension().and_then(|s| s.to_str()) {
        Some("flac") => "audio/flac",
        Some("mp3") => "audio/mpeg",
        Some("ogg") => "audio/ogg",
        Some("m4a") => "audio/mp4",
        _ => "application/octet-stream",
    };

    // Get file metadata
    let file_metadata = tokio::fs::metadata(&track.path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let file_size = file_metadata.len();

    // Parse Range header
    let range_header = headers.get(header::RANGE);

    if let Some(range_value) = range_header {
        // Parse range: "bytes=start-end"
        if let Ok(range_str) = range_value.to_str() {
            if let Some(range) = parse_range(range_str, file_size) {
                return stream_range(&track.path, range.0, range.1, file_size, content_type).await;
            }
        }
    }

    // No range or invalid range - stream entire file
    let mut file = tokio::fs::File::open(&track.path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::debug!("Streaming {} bytes for track {}", buffer.len(), id);

    // Return the file with proper headers
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_LENGTH, file_size.to_string().as_str()),
            (header::ACCEPT_RANGES, "bytes"),
            (
                header::CONTENT_DISPOSITION,
                &format!(
                    "inline; filename=\"{}\"",
                    track.path.file_name().unwrap().to_string_lossy()
                ),
            ),
        ],
        buffer,
    )
        .into_response())
}

/// Parse Range header value
/// Returns (start, end) tuple if valid
fn parse_range(range_str: &str, file_size: u64) -> Option<(u64, u64)> {
    // Expected format: "bytes=start-end" or "bytes=start-" or "bytes=-end"
    if !range_str.starts_with("bytes=") {
        return None;
    }

    let range_part = &range_str[6..]; // Skip "bytes="
    let parts: Vec<&str> = range_part.split('-').collect();

    if parts.len() != 2 {
        return None;
    }

    let start_str = parts[0].trim();
    let end_str = parts[1].trim();

    match (start_str.is_empty(), end_str.is_empty()) {
        (false, false) => {
            // "bytes=start-end"
            let start = start_str.parse::<u64>().ok()?;
            let end = end_str.parse::<u64>().ok()?;
            if start > end || start >= file_size {
                return None;
            }
            Some((start, end.min(file_size - 1)))
        }
        (false, true) => {
            // "bytes=start-" (from start to end of file)
            let start = start_str.parse::<u64>().ok()?;
            if start >= file_size {
                return None;
            }
            Some((start, file_size - 1))
        }
        (true, false) => {
            // "bytes=-end" (last N bytes)
            let suffix_length = end_str.parse::<u64>().ok()?;
            if suffix_length == 0 || suffix_length > file_size {
                return None;
            }
            Some((file_size - suffix_length, file_size - 1))
        }
        (true, true) => None,
    }
}

/// Stream a range of bytes from a file
async fn stream_range(
    path: &std::path::Path,
    start: u64,
    end: u64,
    total_size: u64,
    content_type: &str,
) -> Result<Response, StatusCode> {
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Seek to start position
    file.seek(std::io::SeekFrom::Start(start))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Read the requested range
    let range_length = (end - start + 1) as usize;
    let mut buffer = vec![0u8; range_length];
    file.read_exact(&mut buffer)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::debug!(
        "Streaming range {}-{}/{} ({} bytes)",
        start,
        end,
        total_size,
        range_length
    );

    // Return 206 Partial Content
    Ok((
        StatusCode::PARTIAL_CONTENT,
        [
            (header::CONTENT_TYPE, content_type.to_string()),
            (header::CONTENT_LENGTH, range_length.to_string()),
            (header::ACCEPT_RANGES, "bytes".to_string()),
            (
                header::CONTENT_RANGE,
                format!("bytes {}-{}/{}", start, end, total_size),
            ),
        ],
        buffer,
    )
        .into_response())
}

/// Update track metadata
async fn update_track(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(update): Json<TrackMetadataUpdate>,
) -> Result<Json<Track>, StatusCode> {
    tracing::debug!(
        "Updating track {} with metadata: title={:?}, artist={:?}, album={:?}",
        id,
        update.title,
        update.artist,
        update.album
    );

    let result = state
        .library
        .update_track_metadata(&id, update)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to update track metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        });

    if result.is_ok() {
        tracing::debug!("Successfully updated track {}", id);
    }

    result
}

/// List all albums
async fn list_albums(State(state): State<AppState>) -> Json<Vec<Album>> {
    tracing::debug!("Fetching all albums");
    let albums = state.library.get_albums().await;
    tracing::debug!("Returning {} albums", albums.len());
    Json(albums)
}

/// Get a specific album by name
async fn get_album(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Album>, StatusCode> {
    tracing::debug!("Fetching album: {}", name);
    let result = state
        .library
        .get_album(&name)
        .await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND);

    if result.is_ok() {
        tracing::debug!("Album {} found", name);
    } else {
        tracing::warn!("Album {} not found", name);
    }

    result
}

/// List all artists
async fn list_artists(State(state): State<AppState>) -> Json<Vec<Artist>> {
    tracing::debug!("Fetching all artists");
    let artists = state.library.get_artists().await;
    tracing::debug!("Returning {} artists", artists.len());
    Json(artists)
}

/// Get a specific artist by name
async fn get_artist(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Artist>, StatusCode> {
    tracing::debug!("Fetching artist: {}", name);
    let result = state
        .library
        .get_artist(&name)
        .await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND);

    if result.is_ok() {
        tracing::debug!("Artist {} found", name);
    } else {
        tracing::warn!("Artist {} not found", name);
    }

    result
}

/// Get library statistics
async fn get_stats(State(state): State<AppState>) -> Json<LibraryStats> {
    tracing::debug!("Fetching library statistics");
    let stats = state.library.get_stats().await;
    tracing::debug!(
        "Stats: {} tracks, {} albums, {} artists",
        stats.total_tracks,
        stats.total_albums,
        stats.total_artists
    );
    Json(stats)
}

/// Get cover art for a track
async fn get_cover(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    tracing::debug!("Fetching cover art for track: {}", id);

    let track = state
        .library
        .get_track(&id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    match state.library.get_cover_art(&track.path) {
        Ok(Some(image_data)) => {
            tracing::debug!(
                "Found cover art for track: {} ({} bytes)",
                id,
                image_data.len()
            );

            // Try to determine MIME type from image data
            let mime_type = if image_data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                "image/jpeg"
            } else if image_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                "image/png"
            } else {
                "image/jpeg" // Default to JPEG
            };

            Ok((
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, mime_type),
                    (header::CACHE_CONTROL, "public, max-age=3600"),
                ],
                image_data,
            )
                .into_response())
        }
        Ok(None) => {
            tracing::debug!("No cover art found for track: {}", id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Error reading cover art for track {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Upload cover art for a track
async fn upload_cover(
    State(state): State<AppState>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<Track>, StatusCode> {
    tracing::debug!("Uploading cover art for track: {}", id);

    let mut image_data: Option<Vec<u8>> = None;
    let mut mime_type = "image/jpeg".to_string();

    // Process multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Error reading multipart field: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "image" || name == "cover" {
            if let Some(content_type) = field.content_type() {
                mime_type = content_type.to_string();
            }

            let data = field.bytes().await.map_err(|e| {
                tracing::error!("Error reading image data: {}", e);
                StatusCode::BAD_REQUEST
            })?;

            image_data = Some(data.to_vec());
            break;
        }
    }

    let image_data = image_data.ok_or_else(|| {
        tracing::warn!("No image data found in upload for track: {}", id);
        StatusCode::BAD_REQUEST
    })?;

    // Set the cover art
    state
        .library
        .set_cover_art(&id, image_data, &mime_type)
        .await
        .map_err(|e| {
            tracing::error!("Error setting cover art for track {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Return updated track
    let track = state
        .library
        .get_track(&id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    tracing::debug!("Successfully uploaded cover art for track: {}", id);
    Ok(Json(track))
}

/// Delete cover art for a track
async fn delete_cover(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Track>, StatusCode> {
    tracing::debug!("Deleting cover art for track: {}", id);

    state.library.remove_cover_art(&id).await.map_err(|e| {
        tracing::error!("Error removing cover art for track {}: {}", id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Return updated track
    let track = state
        .library
        .get_track(&id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    tracing::debug!("Successfully deleted cover art for track: {}", id);
    Ok(Json(track))
}

// ========== LYRICS ENDPOINTS ==========

/// Get lyrics for a track
async fn get_lyrics(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Lyric>, StatusCode> {
    tracing::debug!("Fetching lyrics for track: {}", id);

    // Check if track exists
    state
        .library
        .get_track(&id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get lyrics from database
    let lyric = state
        .lyrics_db
        .get_lyric(&id)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching lyrics for track {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::debug!("No lyrics found for track: {}", id);
            StatusCode::NOT_FOUND
        })?;

    tracing::debug!("Successfully fetched lyrics for track: {}", id);
    Ok(Json(lyric))
}

/// Upload or update lyrics for a track
async fn upload_lyrics(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(upload): Json<LyricUpload>,
) -> Result<Json<Lyric>, StatusCode> {
    tracing::debug!("Uploading lyrics for track: {}", id);

    // Check if track exists
    state
        .library
        .get_track(&id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    // Determine format
    let format = if let Some(fmt) = upload.format {
        LyricFormat::from_str(&fmt)
    } else {
        // Auto-detect format based on content
        if upload.content.contains("[00:") || upload.content.contains("[01:") {
            LyricFormat::Lrc
        } else {
            LyricFormat::Plain
        }
    };

    // Save lyrics
    let lyric = state
        .lyrics_db
        .save_lyric(&id, upload.content, format, upload.language, upload.source)
        .await
        .map_err(|e| {
            tracing::error!("Error saving lyrics for track {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Update track's has_lyrics flag
    state.library.update_track_lyrics_status(&id, true).await;

    tracing::debug!("Successfully uploaded lyrics for track: {}", id);
    Ok(Json(lyric))
}

/// Delete lyrics for a track
async fn delete_lyrics(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    tracing::debug!("Deleting lyrics for track: {}", id);

    // Check if track exists
    state
        .library
        .get_track(&id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    // Delete lyrics
    let deleted = state.lyrics_db.delete_lyric(&id).await.map_err(|e| {
        tracing::error!("Error deleting lyrics for track {}: {}", id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !deleted {
        tracing::debug!("No lyrics found to delete for track: {}", id);
        return Err(StatusCode::NOT_FOUND);
    }

    // Update track's has_lyrics flag
    state.library.update_track_lyrics_status(&id, false).await;

    tracing::debug!("Successfully deleted lyrics for track: {}", id);
    Ok(StatusCode::NO_CONTENT)
}

// ========== LYRICS SEARCH ENDPOINTS ==========

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct LyricsSearchQuery {
    q: String,
    provider: String,
    artist: Option<String>,
}

/// Search for lyrics from external providers
async fn search_lyrics(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<LyricsSearchQuery>,
) -> Result<Json<Vec<FetcherSearchResult>>, StatusCode> {
    tracing::debug!(
        "Searching lyrics: query='{}', provider='{}', artist='{:?}'",
        query.q,
        query.provider,
        query.artist
    );

    // Build lyrics query
    let mut lyrics_query = LyricsQuery::new(&query.q);
    if let Some(artist) = query.artist {
        lyrics_query = lyrics_query.with_artist(artist);
    }

    // Select provider and search
    let results = match query.provider.as_str() {
        "netease" => {
            let provider = state.netease_provider.as_ref().ok_or_else(|| {
                tracing::error!("NetEase provider not initialized");
                StatusCode::SERVICE_UNAVAILABLE
            })?;

            provider.search(&lyrics_query).await.map_err(|e| {
                tracing::error!("NetEase search error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
        }
        "qqmusic" => {
            let provider = state.qqmusic_provider.as_ref().ok_or_else(|| {
                tracing::error!("QQ Music provider not initialized");
                StatusCode::SERVICE_UNAVAILABLE
            })?;

            provider.search(&lyrics_query).await.map_err(|e| {
                tracing::error!("QQ Music search error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
        }
        _ => {
            tracing::warn!("Unknown provider: {}", query.provider);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    tracing::debug!("Found {} lyrics search results", results.len());
    Ok(Json(results))
}

/// Fetch lyrics from a specific provider by song ID
async fn fetch_lyrics_from_provider(
    State(state): State<AppState>,
    Path((provider, song_id)): Path<(String, String)>,
) -> Result<Json<LyricsResponse>, StatusCode> {
    tracing::debug!(
        "Fetching lyrics: provider='{}', song_id='{}'",
        provider,
        song_id
    );

    let lyrics = match provider.as_str() {
        "netease" => {
            let provider = state.netease_provider.as_ref().ok_or_else(|| {
                tracing::error!("NetEase provider not initialized");
                StatusCode::SERVICE_UNAVAILABLE
            })?;

            provider.fetch(&song_id).await.map_err(|e| {
                tracing::error!("NetEase fetch error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
        }
        "qqmusic" => {
            let provider = state.qqmusic_provider.as_ref().ok_or_else(|| {
                tracing::error!("QQ Music provider not initialized");
                StatusCode::SERVICE_UNAVAILABLE
            })?;

            provider.fetch(&song_id).await.map_err(|e| {
                tracing::error!("QQ Music fetch error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
        }
        _ => {
            tracing::warn!("Unknown provider: {}", provider);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    tracing::debug!("Successfully fetched lyrics from {}", provider);
    Ok(Json(lyrics))
}

// ========== PLAYLIST ENDPOINTS ==========

/// List all playlists
async fn list_playlists(State(state): State<AppState>) -> Result<Json<Vec<Playlist>>, StatusCode> {
    tracing::debug!("Fetching all playlists");

    let playlists = state.playlist_db.get_playlists().await.map_err(|e| {
        tracing::error!("Error fetching playlists: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::debug!("Returning {} playlists", playlists.len());
    Ok(Json(playlists))
}

/// Get a specific playlist by ID
async fn get_playlist(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Playlist>, StatusCode> {
    tracing::debug!("Fetching playlist: {}", id);

    let playlist = state
        .playlist_db
        .get_playlist(&id)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching playlist {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::debug!("Playlist {} not found", id);
            StatusCode::NOT_FOUND
        })?;

    tracing::debug!(
        "Playlist {} found with {} tracks",
        id,
        playlist.tracks.len()
    );
    Ok(Json(playlist))
}

/// Create a new playlist
async fn create_playlist(
    State(state): State<AppState>,
    Json(create): Json<PlaylistCreate>,
) -> Result<Json<Playlist>, StatusCode> {
    tracing::debug!("Creating playlist: {}", create.name);

    let playlist = state
        .playlist_db
        .create_playlist(create)
        .await
        .map_err(|e| {
            tracing::error!("Error creating playlist: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::debug!("Successfully created playlist: {}", playlist.id);
    Ok(Json(playlist))
}

/// Update a playlist
async fn update_playlist(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(update): Json<PlaylistUpdate>,
) -> Result<Json<Playlist>, StatusCode> {
    tracing::debug!("Updating playlist: {}", id);

    let playlist = state
        .playlist_db
        .update_playlist(&id, update)
        .await
        .map_err(|e| {
            tracing::error!("Error updating playlist {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::debug!("Playlist {} not found", id);
            StatusCode::NOT_FOUND
        })?;

    tracing::debug!("Successfully updated playlist: {}", id);
    Ok(Json(playlist))
}

/// Delete a playlist
async fn delete_playlist(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    tracing::debug!("Deleting playlist: {}", id);

    let deleted = state.playlist_db.delete_playlist(&id).await.map_err(|e| {
        tracing::error!("Error deleting playlist {}: {}", id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if deleted {
        tracing::debug!("Successfully deleted playlist: {}", id);
        Ok(StatusCode::NO_CONTENT)
    } else {
        tracing::debug!("Playlist {} not found", id);
        Err(StatusCode::NOT_FOUND)
    }
}

/// Add a track to a playlist
async fn add_track_to_playlist(
    State(state): State<AppState>,
    Path((playlist_id, track_id)): Path<(String, String)>,
) -> Result<Json<Playlist>, StatusCode> {
    tracing::debug!("Adding track {} to playlist {}", track_id, playlist_id);

    // Verify track exists
    state.library.get_track(&track_id).await.ok_or_else(|| {
        tracing::warn!("Track {} not found", track_id);
        StatusCode::NOT_FOUND
    })?;

    let playlist = state
        .playlist_db
        .add_track_to_playlist(&playlist_id, &track_id)
        .await
        .map_err(|e| {
            tracing::error!("Error adding track to playlist: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::debug!("Playlist {} not found", playlist_id);
            StatusCode::NOT_FOUND
        })?;

    tracing::debug!(
        "Successfully added track {} to playlist {}",
        track_id,
        playlist_id
    );
    Ok(Json(playlist))
}

/// Remove a track from a playlist
async fn remove_track_from_playlist(
    State(state): State<AppState>,
    Path((playlist_id, track_id)): Path<(String, String)>,
) -> Result<Json<Playlist>, StatusCode> {
    tracing::debug!("Removing track {} from playlist {}", track_id, playlist_id);

    let playlist = state
        .playlist_db
        .remove_track_from_playlist(&playlist_id, &track_id)
        .await
        .map_err(|e| {
            tracing::error!("Error removing track from playlist: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::debug!("Playlist {} or track {} not found", playlist_id, track_id);
            StatusCode::NOT_FOUND
        })?;

    tracing::debug!(
        "Successfully removed track {} from playlist {}",
        track_id,
        playlist_id
    );
    Ok(Json(playlist))
}
