<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NSpin, NEmpty, NButton } from 'naive-ui'
import { ArrowLeft } from 'lucide-vue-next'
import type { Artist, Track } from '@/types'
import { fetchArtist } from '@/api/artists'
import { coverUrl } from '@/api/client'
import { formatDuration, formatDurationLong } from '@/utils/format'
import { useArtistsStore } from '@/stores/artists'
import { usePlayerStore } from '@/stores/player'
import { useQueueStore } from '@/stores/queue'

const route = useRoute()
const router = useRouter()
const artistsStore = useArtistsStore()
const player = usePlayerStore()
const queue = useQueueStore()

const artist = ref<Artist | null>(null)
const isLoading = ref(false)
const error = ref<string | null>(null)
let abortController: AbortController | null = null

const artistName = computed(() => decodeURIComponent(route.params.name as string))

// All tracks across all albums, in album order with tracks sorted by track number
const allTracks = computed<Track[]>(() =>
  artist.value?.albums.flatMap((a) => sortAlbumTracks(a.tracks)) ?? [],
)

function sortAlbumTracks(tracks: Track[]): Track[] {
  return [...tracks].sort((a, b) => {
    const na = a.track_number != null ? parseInt(a.track_number, 10) : null
    const nb = b.track_number != null ? parseInt(b.track_number, 10) : null
    if (na == null && nb == null) return 0
    if (na == null) return 1
    if (nb == null) return -1
    return na - nb
  })
}

// Cover: first track with cover art across all albums
const coverTrack = computed(() => allTracks.value.find((t) => t.has_cover) ?? null)

async function load() {
  if (route.name !== 'artist-detail') return
  const name = artistName.value

  const cached = artistsStore.allArtists.find((a) => a.name === name)
  if (cached) {
    artist.value = cached
    return
  }

  abortController?.abort()
  abortController = new AbortController()
  isLoading.value = true
  error.value = null
  try {
    artist.value = await fetchArtist(name, abortController.signal)
  } catch (e) {
    if (e instanceof DOMException && e.name === 'AbortError') return
    error.value = e instanceof Error ? e.message : 'Failed to load artist'
  } finally {
    isLoading.value = false
  }
}

function playAll() {
  const ids = allTracks.value.map((t) => t.id)
  if (!ids.length) return
  queue.setQueue(ids, 0)
  player.playTrack(ids[0]!)
}

function addAllToQueue() {
  queue.addMultiple(allTracks.value.map((t) => t.id))
}

function playAlbum(albumName: string) {
  const rawTracks = artist.value?.albums.find((a) => a.name === albumName)?.tracks ?? []
  const tracks = sortAlbumTracks(rawTracks)
  const ids = tracks.map((t) => t.id)
  if (!ids.length) return
  queue.setQueue(ids, 0)
  player.playTrack(ids[0]!)
}

function playTrack(track: Track, albumName: string) {
  const rawTracks = artist.value?.albums.find((a) => a.name === albumName)?.tracks ?? []
  const tracks = sortAlbumTracks(rawTracks)
  const ids = tracks.map((t) => t.id)
  const idx = ids.indexOf(track.id)
  queue.setQueue(ids, Math.max(idx, 0))
  player.playTrack(track.id)
}

onMounted(load)
watch(() => route.params.name, load)
onUnmounted(() => abortController?.abort())
</script>

<template>
  <div :class="$style.container">
    <div :class="$style.toolbar">
      <NButton quaternary @click="router.back()">
        <template #icon><ArrowLeft :size="16" /></template>
        Artists
      </NButton>
    </div>

    <NSpin :show="isLoading">
      <NEmpty v-if="!isLoading && error" :description="error" style="padding: 60px 0" />
      <NEmpty v-else-if="!isLoading && !artist" description="Artist not found" style="padding: 60px 0" />

      <div v-else-if="artist" :class="$style.content">
        <!-- Header: avatar + info side by side -->
        <div :class="$style.header">
          <div :class="$style.avatar">
            <img
              v-if="coverTrack"
              :src="coverUrl(coverTrack.id)"
              :class="$style.avatarImg"
              :alt="artist.name"
            />
            <div v-else :class="$style.avatarPlaceholder">
              <svg xmlns="http://www.w3.org/2000/svg" width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18V5l12-2v13"></path><circle cx="6" cy="18" r="3"></circle><circle cx="18" cy="16" r="3"></circle></svg>
            </div>
          </div>

          <div :class="$style.artistInfo">
            <div :class="$style.artistLabel">Artist</div>
            <h1 :class="$style.artistName">{{ artist.name }}</h1>
            <div :class="$style.artistMeta">
              <span>{{ artist.album_count }} albums</span>
              <span :class="$style.metaSep">·</span>
              <span>{{ artist.track_count }} tracks</span>
            </div>
            <div :class="$style.actions">
              <NButton type="primary" @click="playAll">
                <template #icon>
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="currentColor" stroke="none"><polygon points="6 3 20 12 6 21 6 3"></polygon></svg>
                </template>
                Play All
              </NButton>
              <NButton @click="addAllToQueue">
                <template #icon>
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 12H3"></path><path d="M16 6H3"></path><path d="M16 18H3"></path><path d="M18 9v6"></path><path d="M21 12h-6"></path></svg>
                </template>
                Add to Queue
              </NButton>
            </div>
          </div>
        </div>

        <!-- Albums with track lists -->
        <div :class="$style.albumList">
          <div
            v-for="album in artist.albums"
            :key="album.name"
            :class="$style.albumSection"
          >
            <div :class="$style.albumHeader">
              <div :class="$style.albumCover">
                <img
                  v-if="album.tracks.find(t => t.has_cover)"
                  :src="coverUrl(album.tracks.find(t => t.has_cover)!.id)"
                  :class="$style.albumCoverImg"
                  loading="lazy"
                />
                <div v-else :class="$style.albumCoverPlaceholder">&#9834;</div>
              </div>
              <div :class="$style.albumInfo">
                <div
                  :class="$style.albumName"
                  @click="router.push({ name: 'album-detail', params: { name: album.name } })"
                >{{ album.name }}</div>
                <div :class="$style.albumMeta">
                  {{ album.track_count }} tracks · {{ formatDurationLong(album.total_duration_secs) }}
                </div>
              </div>
              <button :class="$style.albumPlayBtn" title="Play album" @click="playAlbum(album.name)">
                <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="currentColor" stroke="none"><polygon points="6 3 20 12 6 21 6 3"></polygon></svg>
              </button>
            </div>

            <div :class="$style.trackList">
              <div
                v-for="track in sortAlbumTracks(album.tracks)"
                :key="track.id"
                :class="[$style.trackRow, track.id === player.currentTrackId && $style.trackRowActive]"
                @click="playTrack(track, album.name)"
              >
                <span :class="$style.colNum">{{ track.track_number ?? '—' }}</span>
                <span :class="$style.trackTitle">{{ track.title ?? 'Unknown Title' }}</span>
                <span :class="$style.trackDur">{{ formatDuration(track.duration_secs) }}</span>
                <button
                  :class="$style.addBtn"
                  title="Add to queue"
                  @click.stop="queue.addToQueue(track.id)"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 12H3"></path><path d="M16 6H3"></path><path d="M16 18H3"></path><path d="M18 9v6"></path><path d="M21 12h-6"></path></svg>
                </button>
              </div>
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
  gap: 36px;
}

/* Header */
.header {
  display: flex;
  gap: 32px;
  align-items: flex-end;
}

.avatar {
  flex-shrink: 0;
  width: 200px;
  height: 200px;
  border-radius: 50%;
  overflow: hidden;
  box-shadow: 0 8px 32px var(--app-shadow, rgba(0,0,0,0.3));
}

.avatarImg {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.avatarPlaceholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #1a2332, #0f1419);
  opacity: 0.5;
}

.artistInfo {
  flex: 1;
  min-width: 0;
  padding-bottom: 4px;
}

.artistLabel {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  opacity: 0.5;
  margin-bottom: 8px;
}

.artistName {
  font-size: 32px;
  font-weight: 800;
  line-height: 1.1;
  margin: 0 0 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.artistMeta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  opacity: 0.7;
  margin-bottom: 20px;
}

.metaSep { opacity: 0.4; }

.actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

/* Album sections */
.albumList {
  display: flex;
  flex-direction: column;
  gap: 28px;
}

.albumSection {
  border-top: 1px solid var(--app-border);
  padding-top: 20px;
}

.albumHeader {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 8px;
}

.albumCover {
  flex-shrink: 0;
  width: 52px;
  height: 52px;
  border-radius: 6px;
  overflow: hidden;
}

.albumCoverImg {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.albumCoverPlaceholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #1a2332, #0f1419);
  font-size: 20px;
  opacity: 0.3;
}

.albumInfo {
  flex: 1;
  min-width: 0;
}

.albumName {
  font-size: 15px;
  font-weight: 700;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: pointer;
}

.albumName:hover {
  text-decoration: underline;
  opacity: 0.8;
}

.albumMeta {
  font-size: 12px;
  opacity: 0.5;
  margin-top: 2px;
}

.albumPlayBtn {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: inherit;
  cursor: pointer;
  padding: 0;
  opacity: 0;
  transition: opacity 0.15s, background 0.15s;
}

.albumHeader:hover .albumPlayBtn { opacity: 0.6; }
.albumPlayBtn:hover { opacity: 1 !important; background: rgba(128,128,128,0.15); }

/* Track list */
.trackList {
  display: flex;
  flex-direction: column;
}

.trackRow {
  display: grid;
  grid-template-columns: 36px 1fr 64px 36px;
  align-items: center;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.15s;
}

.trackRow:hover { background: var(--app-hover); }

.trackRowActive { color: var(--n-primary-color, #0066cc); }
.trackRowActive .trackTitle { font-weight: 600; }

.colNum {
  text-align: right;
  opacity: 0.4;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  padding-right: 16px;
}

.trackTitle {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.trackDur {
  font-size: 12px;
  opacity: 0.5;
  font-variant-numeric: tabular-nums;
  text-align: right;
}

.addBtn {
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
  transition: opacity 0.15s, background 0.15s;
}

.trackRow:hover .addBtn { opacity: 0.5; }
.addBtn:hover { opacity: 1 !important; background: rgba(128,128,128,0.15); }

/* Responsive */
@media (max-width: 600px) {
  .container {
    padding: 16px 20px;
  }

  .header {
    flex-direction: column;
    align-items: flex-start;
  }

  .avatar {
    width: 140px;
    height: 140px;
  }

  .artistName {
    font-size: 24px;
  }
}
</style>
