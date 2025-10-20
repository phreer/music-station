use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use tokio::io::AsyncReadExt;
use tower_http::cors::CorsLayer;

use crate::library::{MusicLibrary, Track};

#[derive(Clone)]
pub struct AppState {
    pub library: MusicLibrary,
}

pub fn create_router(library: MusicLibrary) -> Router {
    let state = AppState { library };

    Router::new()
        .route("/", get(root))
        .route("/tracks", get(list_tracks))
        .route("/tracks/:id", get(get_track))
        .route("/stream/:id", get(stream_track))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Root endpoint
async fn root() -> &'static str {
    "Music Station API v0.1.0"
}

/// List all tracks
async fn list_tracks(State(state): State<AppState>) -> Json<Vec<Track>> {
    let tracks = state.library.get_tracks().await;
    Json(tracks)
}

/// Get a specific track by ID
async fn get_track(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Track>, StatusCode> {
    state
        .library
        .get_track(&id)
        .await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// Stream a track by ID
async fn stream_track(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let track = state
        .library
        .get_track(&id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    // Read the file
    let mut file = tokio::fs::File::open(&track.path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
