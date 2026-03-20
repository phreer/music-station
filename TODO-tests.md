# Music Station - Test Implementation Plan

## Current State

- **20 existing tests** focused on lyrics format detection and `music-search-rs` subcrate
- **Zero coverage** on: server (26 endpoints), library (16 methods), playlist DB (8 methods), stats DB (4 methods), audio (31 methods)
- 2 shell scripts (`test_lyrics.sh`, `test_word_level_format.sh`) require a running server — not automated
- No `[dev-dependencies]` in root crate, no mock frameworks, no test fixtures

## Layer 1: Unit Tests (inline `#[cfg(test)]` modules)

Pure functions, no external dependencies, lowest implementation cost.

- [x] `src/server.rs` — `parse_range()`: `bytes=0-499`, `bytes=500-`, `bytes=-500`, boundary cases (start>end, start>=file_size, suffix_length=0, invalid format)
- [x] `src/audio.rs` — `get_audio_file_handler()`: 4 supported formats return `Some`, unsupported returns `None`, case insensitivity
- [x] `src/audio.rs` — `AudioMetadata::update_from_std_key()`: verify StandardTagKey to field mapping

## Layer 2: Database Integration Tests (`#[tokio::test]`, in-memory SQLite)

All 3 DB modules use SQLite. Use `tempfile` crate for test isolation.

**New dev-dependency:** `tempfile = "3"`

- [x] `src/lyrics.rs` — `LyricDatabase` CRUD
  - new → save_lyric → get_lyric (verify fields) → has_lyric → delete_lyric → verify deletion → get_tracks_with_lyrics
  - Format auto-detection on save
  - Overwrite behavior (save twice for same track_id)
- [x] `src/playlist.rs` — `PlaylistDatabase` CRUD
  - create → get_playlists → get_playlist → update → add_track → remove_track → delete → verify cascade
  - Duplicate track prevention
  - Non-existent playlist handling
- [x] `src/stats.rs` — `StatsDatabase`
  - new → increment_play_count (multiple times) → get_play_count → get_all_play_counts
  - Increment from zero
  - Multiple tracks

## Layer 3: HTTP API Integration Tests (`tests/` directory)

Use axum's test pattern (`tower::ServiceExt` + `oneshot`). No real TCP listener needed.

**New dev-dependencies:** `tower = { version = "0.5", features = ["util"] }`, `hyper = "1"`

**Test infrastructure:**
- `tests/api_tests.rs` (or `tests/api/` directory split by module)
- `setup()` helper: temp dir → `MusicLibrary` + 3 DBs → `create_router()` → `Router`

**Endpoint groups to test:**

- [ ] **Root** — GET `/` → 200 + version string
- [ ] **Tracks** — GET `/tracks` empty → `[]`; GET `/tracks/:id` missing → 404
- [ ] **Playlists CRUD** — POST create → GET verify → PUT update → POST add_track → DELETE remove_track → DELETE playlist
- [ ] **Lyrics CRUD** — PUT upload (verify auto-detection) → GET retrieve → DELETE → 404
- [ ] **Stats** — GET `/stats` empty library; POST `/tracks/:id/play` → verify count increment
- [ ] **Albums/Artists** — GET empty → `[]`
- [ ] **Streaming** — Requires test fixture audio files (deferred)
- [ ] **Cover art** — Requires test fixture audio files (deferred)

## Future: Audio Test Fixtures

For `src/audio.rs` format-specific tests:

- [ ] Create `tests/fixtures/` with minimal sample files (< 100KB each): `.flac`, `.mp3`, `.ogg`, `.m4a`
- [ ] Test `parse_metadata()` for each format
- [ ] Test `has_cover_art()` / `get_cover_art()` for files with/without covers
- [ ] Test `write_metadata()` + re-read round-trip (FLAC, MP3, M4A — OGG write is unimplemented)

## Expected Coverage After Full Implementation

| Metric               | Before | After  |
|----------------------|--------|--------|
| Test functions       | 20     | ~80    |
| server.rs coverage   | 0/27   | ~12/27 |
| DB layer coverage    | 0/19   | ~19/19 |
| audio.rs coverage    | 0/31   | ~5/31  |
| library.rs coverage  | 0/16   | ~5/16  |
