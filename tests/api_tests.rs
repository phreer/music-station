//! Layer 3: HTTP API integration tests using tower::ServiceExt::oneshot.
//! No real TCP listener — requests go directly through the Axum router.

use axum::body::Bytes;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use music_station::{
    library::MusicLibrary,
    lyrics::LyricDatabase,
    server::create_router,
    stats::StatsDatabase,
};
use serde_json::Value;
use tower::ServiceExt;

/// Build a fully-wired Router backed by temp directories.
/// Returns the router and the TempDir (must be kept alive).
async fn setup() -> (axum::Router, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join(".music-station");
    tokio::fs::create_dir_all(&base).await.unwrap();

    let library = MusicLibrary::new(dir.path().to_path_buf());
    let lyrics_db = LyricDatabase::new(base.join("lyrics.db")).await.unwrap();
    let playlist_db = music_station::playlist::PlaylistDatabase::new(&base.join("playlists.db"))
        .await
        .unwrap();
    let stats_db = StatsDatabase::new(&base.join("stats.db")).await.unwrap();

    let router = create_router(library, lyrics_db, playlist_db, stats_db);
    (router, dir)
}

/// Helper: send a request and return (status, body bytes).
async fn send(
    router: axum::Router,
    req: Request<axum::body::Body>,
) -> (StatusCode, Bytes) {
    let response = router.oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, body)
}

/// Helper: send a request and parse JSON response.
async fn send_json(router: axum::Router, req: Request<axum::body::Body>) -> (StatusCode, Value) {
    let (status, body) = send(router, req).await;
    let json: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    (status, json)
}

// ---- Root ----

#[tokio::test]
async fn root_returns_version_string() {
    let (app, _dir) = setup().await;
    let req = Request::get("/").body(axum::body::Body::empty()).unwrap();
    let (status, body) = send(app, req).await;

    assert_eq!(status, StatusCode::OK);
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert!(text.contains("Music Station API"));
}

// ---- Tracks ----

#[tokio::test]
async fn list_tracks_empty() {
    let (app, _dir) = setup().await;
    let req = Request::get("/tracks")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, json) = send_json(app, req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json, Value::Array(vec![]));
}

#[tokio::test]
async fn get_track_not_found() {
    let (app, _dir) = setup().await;
    let req = Request::get("/tracks/nonexistent")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = send(app, req).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---- Stats ----

#[tokio::test]
async fn stats_empty_library() {
    let (app, _dir) = setup().await;
    let req = Request::get("/stats")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, json) = send_json(app, req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["total_tracks"], 0);
    assert_eq!(json["total_albums"], 0);
    assert_eq!(json["total_artists"], 0);
}

// ---- Albums & Artists ----

#[tokio::test]
async fn list_albums_empty() {
    let (app, _dir) = setup().await;
    let req = Request::get("/albums")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, json) = send_json(app, req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json, Value::Array(vec![]));
}

#[tokio::test]
async fn list_artists_empty() {
    let (app, _dir) = setup().await;
    let req = Request::get("/artists")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, json) = send_json(app, req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json, Value::Array(vec![]));
}

// ---- Play count ----

#[tokio::test]
async fn play_count_nonexistent_track() {
    let (app, _dir) = setup().await;
    // Incrementing play count for a non-existent track still succeeds at the DB level
    // but the handler checks if the track exists first.
    let req = Request::post("/tracks/nonexistent/play")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = send(app, req).await;

    // Handler returns 404 if track not in library
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---- Playlists CRUD ----

#[tokio::test]
async fn playlists_empty() {
    let (app, _dir) = setup().await;
    let req = Request::get("/playlists")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, json) = send_json(app, req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json, Value::Array(vec![]));
}

#[tokio::test]
async fn playlist_crud_flow() {
    let (app, _dir) = setup().await;

    // Create playlist
    let create_body = serde_json::json!({
        "name": "Test Playlist",
        "description": "A test"
    })
    .to_string();
    let req = Request::post("/playlists")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(create_body))
        .unwrap();
    let (status, json) = send_json(app.clone(), req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["name"], "Test Playlist");
    assert_eq!(json["description"], "A test");
    let playlist_id = json["id"].as_str().unwrap().to_string();

    // Get playlist by ID
    let req = Request::get(&format!("/playlists/{}", playlist_id))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, json) = send_json(app.clone(), req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["name"], "Test Playlist");

    // Update playlist
    let update_body = serde_json::json!({
        "name": "Renamed"
    })
    .to_string();
    let req = Request::put(&format!("/playlists/{}", playlist_id))
        .header("content-type", "application/json")
        .body(axum::body::Body::from(update_body))
        .unwrap();
    let (status, json) = send_json(app.clone(), req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["name"], "Renamed");

    // Add a track — requires track to exist in library, so expect 404
    let req = Request::post(&format!("/playlists/{}/tracks/fake-track-1", playlist_id))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = send(app.clone(), req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Delete playlist
    let req = Request::delete(&format!("/playlists/{}", playlist_id))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = send(app.clone(), req).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify deletion
    let req = Request::get(&format!("/playlists/{}", playlist_id))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = send(app.clone(), req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---- Lyrics CRUD ----

#[tokio::test]
async fn lyrics_crud_requires_track_in_library() {
    let (app, _dir) = setup().await;

    // Get lyrics for non-existent track — 404
    let req = Request::get("/lyrics/fake-id")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = send(app.clone(), req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Upload lyrics — track not in library, should be 404
    let upload_body = serde_json::json!({
        "content": "[00:01.00]Hello world",
        "format": "lrc",
        "language": "en",
        "source": "test"
    })
    .to_string();
    let req = Request::put("/lyrics/fake-id")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(upload_body))
        .unwrap();
    let (status, _) = send(app.clone(), req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Delete lyrics — track not in library, should be 404
    let req = Request::delete("/lyrics/fake-id")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = send(app.clone(), req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---- Playlist not found ----

#[tokio::test]
async fn get_nonexistent_playlist() {
    let (app, _dir) = setup().await;
    let req = Request::get("/playlists/no-such-id")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = send(app, req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_nonexistent_playlist() {
    let (app, _dir) = setup().await;
    let req = Request::delete("/playlists/no-such-id")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = send(app, req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
