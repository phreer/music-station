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

---

## 性能优化计划

> 审计时间：2026-03-21
> 背景：Vue 3 客户端已完成基础重写，TrackList 已使用虚拟滚动、Lucide 按需导入、Vite code splitting，但仍存在以下性能问题需要优化。

### 第一阶段：数据层优化（P0 / P1）

#### P0 — Track 查找改用 Map 索引（O(n) → O(1)）

- **文件**: `web/src/stores/library.ts:25-26`
- **问题**: `findTrack(id)` 使用 `Array.find()` 线性扫描，被 player store、queue store 的 `queueTracks` computed 属性频繁调用。大型曲库下每次调用代价线性增长。
- **方案**: 维护 `trackMap: Map<string, Track>`，在 `loadTracks()` 和 `updateTrackLocally()` 时同步更新，`findTrack()` 改为 `trackMap.get(id)`。
- [ ] 在 `library.ts` 中添加 `trackMap`，修改 `loadTracks()`、`findTrack()`、`updateTrackLocally()`

#### P1 — Album / Artist 数据缓存到 Pinia Store

- **文件**: `web/src/views/AlbumsView.vue:12-21`、`web/src/views/ArtistsView.vue:11-18`
- **问题**: 两个视图在 `onMounted` 时各自 fetch 数据。当前 `AppLayout.vue` 使用 `v-if` 切换视图，每次切换 tab 都会卸载/重挂组件，触发完整的 fetch + re-render 周期。
- **方案**: 创建 `albumsStore` 和 `artistsStore`，首次加载后缓存数据，视图组件判断 store 是否已有数据，有则直接使用，提供手动刷新入口。
- [ ] 新建 `web/src/stores/albums.ts` 和 `web/src/stores/artists.ts`
- [ ] 修改 `AlbumsView.vue`、`ArtistsView.vue` 改用 store 数据

#### P1 — 搜索输入防抖

- **文件**: `web/src/views/TracksView.vue:17-22`、`web/src/stores/library.ts:12-21`
- **问题**: `NInput` 直接 `v-model` 绑定 `library.searchQuery`，每次按键都触发 `filteredTracks` computed 对全部 track 执行 3 个 `toLowerCase().includes()` 操作，并传递给虚拟列表触发重绘。
- **方案**: 添加本地 input ref，通过 200-300ms 防抖后再更新 `searchQuery`。
- [ ] 在 `TracksView.vue` 中实现搜索防抖

---

### 第二阶段：渲染优化（P0 / P1）

#### P0 — 视图切换使用 `<KeepAlive>` 缓存

- **文件**: `web/src/components/layout/AppLayout.vue`
- **问题**: 使用 `v-if/v-else-if` 链切换视图，每次切换 tab 完全销毁并重建整个组件树（触发 `onMounted` → 重新 fetch 数据 → 完整 re-render）。
- **方案**: 用 Vue 内置的 `<KeepAlive>` 包裹动态组件 `<component :is="...">`，设置 `max="5"` 缓存全部视图。
- [ ] 修改 `AppLayout.vue` 使用 `<KeepAlive>` + 动态组件

#### P1 — Album / Artist Grid 虚拟化或分页

- **文件**: `web/src/components/albums/AlbumGrid.vue`、`web/src/components/artists/ArtistGrid.vue`
- **问题**: 两个 Grid 均使用 `NGrid + v-for` 一次性渲染全部卡片，无虚拟化。大型曲库可能有数百个专辑/艺术家，每张卡片含封面 `<img>`，首次渲染触发大量并发 HTTP 图片请求和 DOM 节点。
- **方案（按优先级）**:
  - 方案 A（推荐）：使用 Intersection Observer 实现 infinite scroll，每批渲染 50 个
  - 方案 B：使用 `NVirtualList` 做一维虚拟化（需将 grid 行转换为列表项）
  - 方案 C：简单分页（每页 48 个）
- [ ] 实现 `AlbumGrid.vue` 的分批/虚拟渲染
- [ ] 实现 `ArtistGrid.vue` 的分批/虚拟渲染

#### P2 — 缓存 handlePlay 中的 track ID 列表

- **文件**: `web/src/components/tracks/TrackList.vue:23-28`
- **问题**: `handlePlay()` 每次点击时执行 `props.tracks.map(t => t.id)` 构建完整 ID 数组，并用 `ids.indexOf(track.id)` 做线性定位，对数千首曲目的列表是一次完整 O(n) 操作。
- **方案**: 将 ID 数组和 ID→index 映射提升为 `computed`，仅在 `props.tracks` 变化时重新计算。
- [ ] 在 `TrackList.vue` 中用 `computed` 缓存 `trackIds` 和 `trackIdIndexMap`

---

### 第三阶段：播放体验优化（P2）

#### P2 — 歌词时间同步节流

- **文件**: `web/src/stores/player.ts:34-36`、`web/src/components/player/MusicPlayer.vue:48-55`
- **问题**: `timeupdate` 事件以 4-15 Hz 频率触发，每次都更新 `player.currentTime`，进而触发 watch → `lyrics.updateCurrentTime()` → Vue reactivity 传播链。播放期间每秒最多触发 15 次。
- **方案**: 在 `player.ts` 的 `initAudio()` 中对 `timeupdate` 处理函数添加节流（250ms / 4 Hz）。
- [ ] 在 `player.ts` 的 `timeupdate` 事件处理中添加节流

#### P2 — 歌词时间查找改用增量查找 + 二分搜索

- **文件**: `web/src/stores/lyrics.ts:51-63`
- **问题**: `updateCurrentTime()` 每次从头线性扫描 `parsedLines` 数组。正常顺序播放时，当前行几乎总是上一行的下一行，完整线性扫描是冗余操作。
- **方案**: 正常播放从 `currentLineIndex` 开始增量前向查找；seek 跳转时使用二分搜索。
- [ ] 优化 `lyrics.ts` 中的 `updateCurrentTime()` 实现

---

### 第四阶段：网络与加载优化（P2 / P3）

#### P2 — API 请求添加取消支持和错误重试

- **文件**: `web/src/api/client.ts`
- **问题**: API 客户端无 `AbortController` 支持，视图销毁时无法取消进行中的请求，可能导致过时响应覆盖新数据（race condition）。同时无重试机制，瞬时网络错误直接失败。
- **方案**:
  - 为 `request()` 添加可选 `signal: AbortSignal` 参数
  - Store 的 `loadXxx()` 方法在新请求发起时 abort 上一次未完成的请求
  - 网络错误（非 4xx/5xx）自动重试 1-2 次（指数退避）
- [ ] 修改 `api/client.ts` 支持 `AbortSignal`
- [ ] 在各 store 的加载方法中管理 AbortController 生命周期

#### P3 — 视图组件懒加载

- **文件**: `web/src/components/layout/AppLayout.vue`
- **问题**: `AppLayout.vue` 静态 import 所有 5 个视图，全部被打包进初始 chunk，增加首屏加载时间。
- **方案**: 改用 `defineAsyncComponent(() => import('./views/Xxx.vue'))` 实现懒加载，非首屏视图在首次访问时才下载。
- [ ] 修改 `AppLayout.vue` 将各视图改为 `defineAsyncComponent` 懒加载

---

### 第五阶段：架构改进（P3）

#### P3 — 引入 vue-router

- **问题**: 当前用 Pinia store + `v-if` 手动管理视图切换，无 URL 深度链接、无浏览器前进/后退、无法书签收藏特定视图、所有视图代码强制同步加载。
- **方案**: 引入 `vue-router`，每个视图对应一个路由，天然支持路由级懒加载，并可配合 `<RouterView>` + `<KeepAlive>` 作为第二阶段 P0 方案的标准替代。
- [ ] 安装 `vue-router`，创建 `web/src/router/index.ts`
- [ ] 迁移视图切换逻辑（AppNav、AppLayout、ui store）

#### P3 — Naive UI bundle 体积分析与优化

- **文件**: `web/vite.config.ts`
- **问题**: 需确认 naive-ui 的 tree-shaking 是否有效。若 naive-ui chunk 超过 500KB（gzip），应考虑 `unplugin-auto-import` + `unplugin-vue-components` 实现真正的按需导入。
- [ ] 运行 `npm run build` 并分析 bundle 体积（`npx vite-bundle-visualizer`）
- [ ] 若 naive-ui chunk 过大，集成 `unplugin-vue-components`

---

### 优先级总览

| 优先级 | 任务 | 预估工作量 | 主要收益 |
|--------|------|-----------|---------|
| **P0** | Track Map 索引 | ~30 min | 所有 track 查找 O(1)，影响广泛 |
| **P0** | KeepAlive 缓存视图 | ~30 min | 消除 tab 切换时的重复 fetch 和 re-render |
| **P1** | Album/Artist 数据缓存 | ~1-2 h | 避免重复 API 请求 |
| **P1** | Album/Artist Grid 虚拟化/分页 | ~2-3 h | 大型曲库渲染性能显著提升 |
| **P1** | 搜索防抖 | ~30 min | 避免高频过滤计算 |
| **P2** | 歌词同步节流 | ~30 min | 减少播放期间的 reactivity 更新 |
| **P2** | 缓存 queue ID 列表 | ~15 min | 避免点击时的冗余 map 操作 |
| **P2** | API 请求取消与重试 | ~1-2 h | 提升可靠性，避免 race condition |
| **P2** | 歌词二分搜索 | ~15 min | 歌词同步微优化 |
| **P3** | 视图懒加载 | ~30 min | 减少首屏 bundle 体积 |
| **P3** | Naive UI bundle 分析 | ~1 h | 减少整体 bundle 体积 |
| **P3** | 引入 vue-router | ~3-4 h | 深度链接、浏览器导航、标准懒加载方案 |

**总预估工作量**：约 10-15 小时

**建议实施顺序**：P0（改动最小、收益最明显）→ P1 → P2 → P3
