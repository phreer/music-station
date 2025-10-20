use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use tokio::io::AsyncReadExt;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::library::{MusicLibrary, Track, TrackMetadataUpdate};

#[derive(Clone)]
pub struct AppState {
    pub library: MusicLibrary,
}

pub fn create_router(library: MusicLibrary) -> Router {
    let state = AppState { library };

    // Serve static files from ./static directory
    let static_service = ServeDir::new("static");

    Router::new()
        .route("/", get(root))
        .route("/tracks", get(list_tracks))
        .route("/tracks/:id", get(get_track).put(update_track))
        .route("/stream/:id", get(stream_track))
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

/// Stream a track by ID
async fn stream_track(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    tracing::debug!("Streaming track with id: {}", id);
    let track = state
        .library
        .get_track(&id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    
    tracing::debug!("Streaming file: {}", track.path.display());

    // Read the file
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
            (header::CONTENT_TYPE, "audio/flac"),
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

/// Update track metadata
async fn update_track(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(update): Json<TrackMetadataUpdate>,
) -> Result<Json<Track>, StatusCode> {
    tracing::debug!("Updating track {} with metadata: title={:?}, artist={:?}, album={:?}", 
        id, update.title, update.artist, update.album);
    
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
