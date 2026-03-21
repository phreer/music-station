# Web Client Rewrite Plan

## Decision

Rewrite the web client from scratch using a modern frontend stack. The legacy version is preserved at `/web-legacy/index.html`.

## Tech Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Build Tool | Vite 6 | Fast HMR, native ESM, zero-config TS support |
| Framework | Vue 3 (Composition API + `<script setup>`) | SFC component model, gentle learning curve, strong ecosystem |
| Language | TypeScript | Type safety, IDE support, catch errors at compile time |
| UI Library | Naive UI | Dark theme support, rich component set, good for desktop-style apps |
| State Mgmt | Pinia | Official Vue store, simple API, TypeScript-first |
| Icons | lucide-vue-next | Tree-shakable, replaces CDN-loaded Lucide |
| CSS | CSS Modules via `<style module>` + Naive UI theme | Scoped styles, no naming collisions |
| HTTP | Native fetch (wrapped) | No extra dependency needed |

## Source Layout

```
web/                              # Frontend project root
├── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts                # build.outDir -> ../static, base -> /web/
├── src/
│   ├── main.ts                   # createApp, install Pinia + Naive UI
│   ├── App.vue                   # Root: layout + view switching
│   ├── types/
│   │   └── index.ts              # Track, Album, Artist, Playlist, Lyrics, Stats
│   ├── api/
│   │   ├── client.ts             # Base fetch wrapper (error handling, base URL)
│   │   ├── tracks.ts             # GET/PUT /tracks, POST /tracks/:id/play
│   │   ├── streaming.ts          # URL builders for /stream/:id, /cover/:id
│   │   ├── lyrics.ts             # GET/PUT/DELETE /lyrics, search, fetch
│   │   ├── albums.ts             # GET /albums, /albums/:name
│   │   ├── artists.ts            # GET /artists, /artists/:name
│   │   ├── playlists.ts          # CRUD /playlists
│   │   └── stats.ts              # GET /stats
│   ├── stores/
│   │   ├── player.ts             # Playback state: current track, playing, progress, volume
│   │   ├── queue.ts              # Play queue: list, index, add/remove/reorder
│   │   ├── library.ts            # Track data: all tracks, search/filter, pagination
│   │   ├── playlists.ts          # Playlist CRUD state
│   │   ├── lyrics.ts             # Lyrics state: content, sync, search, auto-fetch
│   │   └── ui.ts                 # UI state: current view, theme, sidebar visibility
│   ├── composables/
│   │   ├── useAudioPlayer.ts     # HTMLAudioElement wrapper
│   │   ├── useLyricsSync.ts      # Time-based lyrics line highlighting
│   │   ├── useLrcParser.ts       # LRC / word-level lyrics parsing
│   │   ├── useInfiniteScroll.ts  # Virtual scroll / lazy loading
│   │   └── useResizable.ts       # Drag-to-resize sidebar
│   ├── components/
│   │   ├── layout/
│   │   │   ├── AppHeader.vue
│   │   │   ├── AppNav.vue
│   │   │   └── AppLayout.vue
│   │   ├── player/
│   │   │   ├── MusicPlayer.vue
│   │   │   ├── PlayerControls.vue
│   │   │   ├── PlayerProgress.vue
│   │   │   ├── PlayerVolume.vue
│   │   │   └── PlayerInfo.vue
│   │   ├── queue/
│   │   │   ├── QueuePanel.vue
│   │   │   ├── QueueItem.vue
│   │   │   └── QueueToggle.vue
│   │   ├── tracks/
│   │   │   ├── TrackList.vue
│   │   │   ├── TrackRow.vue
│   │   │   ├── TrackActions.vue
│   │   │   └── TrackSearch.vue
│   │   ├── albums/
│   │   │   ├── AlbumGrid.vue
│   │   │   └── AlbumCard.vue
│   │   ├── artists/
│   │   │   ├── ArtistGrid.vue
│   │   │   └── ArtistCard.vue
│   │   ├── playlists/
│   │   │   ├── PlaylistGrid.vue
│   │   │   ├── PlaylistCard.vue
│   │   │   └── PlaylistTrackList.vue
│   │   ├── lyrics/
│   │   │   ├── LyricsSidebar.vue
│   │   │   ├── LyricsLine.vue
│   │   │   └── LyricsDisplay.vue
│   │   ├── stats/
│   │   │   └── StatsView.vue
│   │   └── modals/
│   │       ├── EditTrackModal.vue
│   │       ├── LyricsModal.vue
│   │       ├── LyricsSearchModal.vue
│   │       ├── AutoFetchModal.vue
│   │       ├── CreatePlaylistModal.vue
│   │       └── AddToPlaylistModal.vue
│   ├── views/
│   │   ├── TracksView.vue
│   │   ├── AlbumsView.vue
│   │   ├── ArtistsView.vue
│   │   ├── PlaylistsView.vue
│   │   └── StatsView.vue
│   └── utils/
│       ├── format.ts             # formatDuration, formatFileSize, formatTimestamp
│       └── html.ts               # escapeHtml (backup; Vue templates auto-escape)
```

## State Management (Pinia Stores)

Replaces the original 33 global mutable variables with 6 typed stores:

| Store | Responsibility | Original Globals |
|-------|---------------|-----------------|
| `usePlayerStore` | Current track, isPlaying, volume, progress | `isPlaying`, `currentTrackIndex`, `playlist` |
| `useQueueStore` | Queue list, index, visibility | `playQueue`, `queueIndex`, `isQueueVisible` |
| `useLibraryStore` | All tracks, search, filter, pagination | `fullTracks`, `tracks`, `searchQuery`, `filteredTracks`, `currentPage`, `pageSize`, `totalTracks` |
| `usePlaylistStore` | Playlists, CRUD operations | `playlists`, `currentPlaylistId`, `trackToAdd` |
| `useLyricsStore` | Lyrics content, sync state, search, auto-fetch | `currentLyrics`, `lyricsLines`, `currentLyricsIndex`, `autoFetchState` |
| `useUiStore` | Active view, theme, sidebar state | `currentView`, `lyricsVisible`, `hasLyricsAvailable` |

## Build Integration

- `vite build` outputs to `../static/` so Rust backend serves it at `/web` with no config changes
- Development: `vite dev` on port 5173, proxy API requests to Rust backend on port 3000
- Legacy version: Rust backend serves `static-legacy/` at `/web-legacy`

## Implementation Phases

### Phase 1: Scaffold + Base Layout
- Initialize Vite + Vue 3 + TS project in `web/`
- Install dependencies (naive-ui, pinia, lucide-vue-next)
- Configure Vite (outDir, base, proxy)
- Define TypeScript types (`types/index.ts`)
- Implement API layer (`api/`)
- Create Pinia store skeletons
- Implement `App.vue` layout + `AppHeader` + `AppNav`
- Configure Naive UI theme (dark/light toggle)

### Phase 2: Core — Track List + Player
- `useLibraryStore` + `TracksView` + `TrackList` + `TrackRow`
- `usePlayerStore` + `useAudioPlayer` composable
- `MusicPlayer` and sub-components (Controls, Progress, Volume, Info)
- Search/filter (`TrackSearch`)
- Milestone: load track list, play audio, search works

### Phase 3: Queue + Playlists
- `useQueueStore` + `QueuePanel` + `QueueItem` (drag-to-reorder)
- `usePlaylistStore` + `PlaylistGrid` + `PlaylistCard`
- `CreatePlaylistModal` + `AddToPlaylistModal`
- Milestone: queue operations and playlist CRUD work

### Phase 4: Albums + Artists + Stats
- `AlbumsView` + `AlbumGrid` + `AlbumCard`
- `ArtistsView` + `ArtistGrid` + `ArtistCard`
- `StatsView`
- Milestone: all 5 views functional

### Phase 5: Lyrics System
- `useLyricsStore` + `useLrcParser` + `useLyricsSync`
- `LyricsSidebar` (synchronized player lyrics)
- `LyricsModal` (view/edit)
- `LyricsSearchModal` (online search)
- `AutoFetchModal` (batch fetch)
- Milestone: lyrics display, sync highlight, search, auto-fetch

### Phase 6: Edit + Polish
- `EditTrackModal` (metadata editing, cover upload/delete, custom fields)
- Resizable sidebar (`useResizable`)
- URL routing (hash-based)
- Persist preferences to localStorage (theme, volume, sidebar width)
- Unified error handling via Naive UI Message/Notification
- Responsive design improvements

### Phase 7: Cleanup + Integration
- `vite build` and verify output in `static/`
- Verify Rust backend serves new frontend
- Update `.gitignore` (node_modules, build cache)
- Update project documentation

## Improvements Over Legacy

| Issue | Legacy | New |
|-------|--------|-----|
| XSS | innerHTML with no escaping | Vue template auto-escaping |
| Code structure | 1 x 3000-line JS file | ~40 components, each < 200 lines |
| State | 33 global `let` variables | 6 typed Pinia stores |
| CSS | 75+ duplicate selectors, 3500 lines | Scoped CSS Modules + Naive UI theme |
| Types | None | Full TypeScript |
| Dark theme | Hardcoded colors everywhere | Naive UI theme system |
| Components | None; global functions | Vue SFC with clear hierarchy |
| Events | Inline onclick + global functions | Vue event binding + component communication |
| Accessibility | Almost none | Naive UI built-in ARIA support |
| URL routing | None (refresh resets view) | Hash-based routing with deep links |
