<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NEmpty, NSpin } from 'naive-ui'
import { ArrowLeft, ListPlus, Music, Play, Trash2 } from 'lucide-vue-next'
import type { Playlist, Track } from '@/types'
import { coverUrl } from '@/api/client'
import { fetchPlaylist } from '@/api/playlists'
import { formatDuration, formatDurationLong } from '@/utils/format'
import { useLibraryStore } from '@/stores/library'
import { usePlayerStore } from '@/stores/player'
import { usePlaylistStore } from '@/stores/playlists'
import { useQueueStore } from '@/stores/queue'
import TrackFavoriteButton from '@/components/tracks/TrackFavoriteButton.vue'

const route = useRoute()
const router = useRouter()
const library = useLibraryStore()
const player = usePlayerStore()
const playlistStore = usePlaylistStore()
const queue = useQueueStore()

const playlist = ref<Playlist | null>(null)
const isLoadingPlaylist = ref(false)
const error = ref<string | null>(null)
let abortController: AbortController | null = null

const playlistId = computed(() => route.params.id as string)

const tracks = computed<Track[]>(() => {
  if (!playlist.value) return []
  return playlist.value.tracks
    .map((id) => library.findTrack(id))
    .filter((track): track is Track => track !== undefined)
})

const coverTrackIds = computed(() =>
  tracks.value.filter((track) => track.has_cover).slice(0, 4).map((track) => track.id),
)

const totalDuration = computed(() =>
  tracks.value.reduce((sum, track) => sum + (track.duration_secs ?? 0), 0),
)

const unresolvedTrackCount = computed(() => {
  if (!playlist.value) return 0
  return playlist.value.tracks.length - tracks.value.length
})

const isLoading = computed(() => isLoadingPlaylist.value || library.isLoading)

async function load() {
  if (route.name !== 'playlist-detail') return
  const id = playlistId.value

  abortController?.abort()
  abortController = new AbortController()
  isLoadingPlaylist.value = true
  error.value = null

  try {
    const cached = playlistStore.findPlaylist(id)
    playlist.value = cached ?? await fetchPlaylist(id, abortController.signal)

    if (playlist.value.tracks.length > 0 && library.totalTracks === 0) {
      await library.loadTracks()
    }
  } catch (e) {
    if (e instanceof DOMException && e.name === 'AbortError') return
    error.value = e instanceof Error ? e.message : 'Failed to load playlist'
  } finally {
    isLoadingPlaylist.value = false
  }
}

function playPlaylist() {
  const ids = tracks.value.map((track) => track.id)
  if (!ids.length) return
  queue.setQueue(ids, 0)
  player.playTrack(ids[0]!)
}

function addPlaylistToQueue() {
  queue.addMultiple(tracks.value.map((track) => track.id))
}

function playTrack(track: Track) {
  const ids = tracks.value.map((item) => item.id)
  const idx = ids.indexOf(track.id)
  queue.setQueue(ids, Math.max(idx, 0))
  player.playTrack(track.id)
}

async function removeTrack(trackId: string) {
  if (!playlist.value) return
  await playlistStore.removeTrack(playlist.value.id, trackId)
  playlist.value.tracks = playlist.value.tracks.filter((id) => id !== trackId)
}

onMounted(load)
watch(() => route.params.id, load)
onUnmounted(() => abortController?.abort())
</script>

<template>
  <div :class="$style.container">
    <div :class="$style.toolbar">
      <NButton quaternary @click="router.back()">
        <template #icon><ArrowLeft :size="16" /></template>
        Playlists
      </NButton>
    </div>

    <NSpin :show="isLoading">
      <NEmpty v-if="!isLoading && error" :description="error" style="padding: 60px 0" />
      <NEmpty v-else-if="!isLoading && !playlist" description="Playlist not found" style="padding: 60px 0" />

      <div v-else-if="playlist" :class="$style.content">
        <div :class="$style.header">
          <div :class="$style.coverWrapper">
            <div v-if="coverTrackIds.length > 0" :class="$style.coverGrid">
              <img
                v-for="id in coverTrackIds"
                :key="id"
                :src="coverUrl(id)"
                :class="$style.coverGridImg"
                loading="lazy"
                :alt="playlist.name"
              />
            </div>
            <div v-else :class="$style.coverPlaceholder">
              <Music :size="68" :stroke-width="1.4" />
            </div>
          </div>

          <div :class="$style.playlistInfo">
            <div :class="$style.playlistLabel">Playlist</div>
            <h1 :class="$style.playlistName">{{ playlist.name }}</h1>
            <div v-if="playlist.description" :class="$style.description">
              {{ playlist.description }}
            </div>
            <div :class="$style.playlistMeta">
              <span>{{ playlist.tracks.length }} tracks</span>
              <span :class="$style.metaSep">·</span>
              <span>{{ formatDurationLong(totalDuration) }}</span>
              <span v-if="unresolvedTrackCount > 0" :class="$style.missingTracks">
                {{ unresolvedTrackCount }} unavailable
              </span>
            </div>

            <div :class="$style.actions">
              <NButton type="primary" :disabled="tracks.length === 0" @click="playPlaylist">
                <template #icon><Play :size="14" fill="currentColor" /></template>
                Play All
              </NButton>
              <NButton :disabled="tracks.length === 0" @click="addPlaylistToQueue">
                <template #icon><ListPlus :size="14" /></template>
                Add to Queue
              </NButton>
            </div>
          </div>
        </div>

        <div :class="$style.trackList">
          <div :class="$style.trackListHeader">
            <span :class="$style.colNum">#</span>
            <span :class="$style.colTitle">Title</span>
            <span :class="$style.colAlbum">Album</span>
            <span :class="$style.colDur">Duration</span>
          </div>

          <NEmpty
            v-if="!isLoading && tracks.length === 0"
            description="No tracks in this playlist"
            style="padding: 48px 0"
          />

          <div
            v-for="(track, index) in tracks"
            :key="track.id"
            :class="[$style.trackRow, track.id === player.currentTrackId && $style.trackRowActive]"
            @click="playTrack(track)"
          >
            <span :class="$style.colNum">{{ index + 1 }}</span>
            <div :class="$style.colTitle">
              <span :class="$style.trackTitle">{{ track.title ?? 'Unknown Title' }}</span>
              <span
                v-if="track.artist"
                :class="[$style.trackArtist, $style.navLink]"
                @click.stop="router.push({ name: 'artist-detail', params: { name: track.artist } })"
              >{{ track.artist }}</span>
            </div>
            <span
              :class="[$style.colAlbum, track.album && $style.navLink]"
              @click.stop="track.album && router.push({ name: 'album-detail', params: { name: track.album } })"
            >{{ track.album ?? '-' }}</span>
            <span :class="$style.colDur">{{ formatDuration(track.duration_secs) }}</span>
            <div :class="$style.rowActions">
              <TrackFavoriteButton :track-id="track.id" />
              <button
                :class="$style.addBtn"
                title="Add to queue"
                @click.stop="queue.addToQueue(track.id)"
              >
                <ListPlus :size="13" />
              </button>
              <button
                :class="$style.removeBtn"
                title="Remove from playlist"
                @click.stop="removeTrack(track.id)"
              >
                <Trash2 :size="13" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </NSpin>
  </div>
</template>

<style module>
.container {
  padding: 32px 48px;
  max-width: 960px;
  margin: 0 auto;
}

.toolbar {
  margin-bottom: 24px;
}

.content {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.header {
  display: flex;
  gap: 32px;
  align-items: flex-end;
}

.coverWrapper {
  flex-shrink: 0;
  width: 220px;
  height: 220px;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 8px 32px var(--app-shadow, rgba(0,0,0,0.3));
}

.coverGrid {
  width: 100%;
  height: 100%;
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
  gap: 2px;
  background: var(--app-border);
}

.coverGridImg {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.coverPlaceholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #1a2332, #2a3f55);
  color: #fff;
  opacity: 0.35;
}

.playlistInfo {
  flex: 1;
  min-width: 0;
  padding-bottom: 4px;
}

.playlistLabel {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  opacity: 0.5;
  margin-bottom: 8px;
}

.playlistName {
  font-size: 32px;
  font-weight: 800;
  line-height: 1.1;
  margin: 0 0 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.description {
  font-size: 14px;
  line-height: 1.5;
  opacity: 0.72;
  margin-bottom: 12px;
  max-width: 620px;
}

.playlistMeta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  opacity: 0.7;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.metaSep { opacity: 0.4; }
.missingTracks { color: var(--app-error); }

.actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.trackList {
  border-top: 1px solid var(--app-border);
}

.trackListHeader,
.trackRow {
  display: grid;
  grid-template-columns: 42px minmax(0, 1.6fr) minmax(120px, 0.9fr) 80px 112px;
  align-items: center;
  gap: 12px;
}

.trackListHeader {
  padding: 10px 12px;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  opacity: 0.45;
}

.trackRow {
  position: relative;
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.trackRow:hover { background: var(--app-hover); }
.trackRowActive { color: var(--n-primary-color, #0066cc); font-weight: 600; }

.colNum {
  font-variant-numeric: tabular-nums;
  opacity: 0.55;
}

.colTitle {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.trackTitle,
.trackArtist,
.colAlbum {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.trackTitle { font-weight: 500; }
.trackArtist { font-size: 12px; opacity: 0.58; }
.colAlbum { opacity: 0.7; }
.colDur {
  justify-self: end;
  opacity: 0.55;
  font-variant-numeric: tabular-nums;
}

.navLink { cursor: pointer; }
.navLink:hover { color: var(--n-primary-color, #0066cc); text-decoration: underline; }

.rowActions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
}

.addBtn,
.removeBtn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: inherit;
  cursor: pointer;
  padding: 0;
  opacity: 0;
  transition: opacity 0.15s, background 0.15s, color 0.15s;
}

.trackRow :global(.track-favorite-button) { opacity: 0; }
.trackRow:hover :global(.track-favorite-button),
.trackRow:hover .addBtn,
.trackRow:hover .removeBtn { opacity: 0.65; }

.addBtn:hover { opacity: 1 !important; background: var(--app-active-bg); }
.removeBtn:hover { opacity: 1 !important; background: var(--app-danger-bg); color: var(--app-danger); }

@media (max-width: 760px) {
  .container { padding: 24px 20px; }
  .header { align-items: flex-start; gap: 18px; }
  .coverWrapper { width: 120px; height: 120px; }
  .playlistName { font-size: 24px; white-space: normal; }
  .colAlbum { display: none; }
  .trackListHeader,
  .trackRow { grid-template-columns: 32px minmax(0, 1fr) 64px 100px; gap: 8px; }
  .trackListHeader .colAlbum,
  .trackRow .colAlbum { display: none; }
}

@media (max-width: 520px) {
  .header { flex-direction: column; }
  .coverWrapper { width: 160px; height: 160px; }
  .trackListHeader,
  .trackRow { grid-template-columns: 28px minmax(0, 1fr) 86px; }
  .colDur { display: none; }
}
</style>
