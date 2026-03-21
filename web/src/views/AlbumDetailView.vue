<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NSpin, NEmpty, NButton } from 'naive-ui'
import { ArrowLeft } from 'lucide-vue-next'
import type { Album, Track } from '@/types'
import { fetchAlbum } from '@/api/albums'
import { coverUrl } from '@/api/client'
import { formatDuration, formatDurationLong } from '@/utils/format'
import { useAlbumsStore } from '@/stores/albums'
import { usePlayerStore } from '@/stores/player'
import { useQueueStore } from '@/stores/queue'

const route = useRoute()
const router = useRouter()
const albumsStore = useAlbumsStore()
const player = usePlayerStore()
const queue = useQueueStore()

const album = ref<Album | null>(null)
const isLoading = ref(false)
const error = ref<string | null>(null)
let abortController: AbortController | null = null

const albumName = computed(() => decodeURIComponent(route.params.name as string))

const coverTrack = computed(() => album.value?.tracks.find((t) => t.has_cover) ?? null)

const albumYear = computed(() => {
  const year = album.value?.tracks.find((t) => t.year)?.year
  return year ?? null
})

async function load() {
  if (route.name !== 'album-detail') return
  const name = albumName.value

  // Use cached data from store if available
  const cached = albumsStore.allAlbums.find((a) => a.name === name)
  if (cached) {
    album.value = cached
    return
  }

  abortController?.abort()
  abortController = new AbortController()
  isLoading.value = true
  error.value = null
  try {
    album.value = await fetchAlbum(name, abortController.signal)
  } catch (e) {
    if (e instanceof DOMException && e.name === 'AbortError') return
    error.value = e instanceof Error ? e.message : 'Failed to load album'
  } finally {
    isLoading.value = false
  }
}

function playAlbum() {
  if (!album.value) return
  const ids = album.value.tracks.map((t) => t.id)
  if (ids.length === 0) return
  queue.setQueue(ids, 0)
  player.playTrack(ids[0]!)
}

function addAlbumToQueue() {
  if (!album.value) return
  queue.addMultiple(album.value.tracks.map((t) => t.id))
}

function playTrack(track: Track) {
  if (!album.value) return
  const ids = album.value.tracks.map((t) => t.id)
  const idx = ids.indexOf(track.id)
  queue.setQueue(ids, idx)
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
        Albums
      </NButton>
    </div>

    <NSpin :show="isLoading">
      <NEmpty v-if="!isLoading && error" :description="error" style="padding: 60px 0" />
      <NEmpty v-else-if="!isLoading && !album" description="Album not found" style="padding: 60px 0" />

      <div v-else-if="album" :class="$style.content">
        <!-- Album header: cover + info side by side -->
        <div :class="$style.header">
          <div :class="$style.coverWrapper">
            <img
              v-if="coverTrack"
              :src="coverUrl(coverTrack.id)"
              :class="$style.coverImg"
              :alt="album.name"
            />
            <div v-else :class="$style.coverPlaceholder">&#9834;</div>
          </div>

          <div :class="$style.albumInfo">
            <div :class="$style.albumLabel">Album</div>
            <h1 :class="$style.albumName">{{ album.name }}</h1>
            <div :class="$style.albumMeta">
              <span v-if="album.artist" :class="$style.artistName">{{ album.artist }}</span>
              <span v-if="albumYear" :class="$style.metaSep">·</span>
              <span v-if="albumYear">{{ albumYear }}</span>
              <span :class="$style.metaSep">·</span>
              <span>{{ album.track_count }} tracks</span>
              <span :class="$style.metaSep">·</span>
              <span>{{ formatDurationLong(album.total_duration_secs) }}</span>
            </div>

            <div :class="$style.actions">
              <NButton type="primary" @click="playAlbum">
                <template #icon>
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="currentColor" stroke="none"><polygon points="6 3 20 12 6 21 6 3"></polygon></svg>
                </template>
                Play All
              </NButton>
              <NButton @click="addAlbumToQueue">
                <template #icon>
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 12H3"></path><path d="M16 6H3"></path><path d="M16 18H3"></path><path d="M18 9v6"></path><path d="M21 12h-6"></path></svg>
                </template>
                Add to Queue
              </NButton>
            </div>
          </div>
        </div>

        <!-- Track list -->
        <div :class="$style.trackList">
          <div :class="$style.trackListHeader">
            <span :class="$style.colNum">#</span>
            <span :class="$style.colTitle">Title</span>
            <span :class="$style.colDur">Duration</span>
          </div>
          <div
            v-for="track in album.tracks"
            :key="track.id"
            :class="[$style.trackRow, track.id === player.currentTrackId && $style.trackRowActive]"
            @click="playTrack(track)"
          >
            <span :class="$style.colNum">{{ track.track_number ?? '—' }}</span>
            <div :class="$style.colTitle">
              <span :class="$style.trackTitle">{{ track.title ?? 'Unknown Title' }}</span>
              <span v-if="track.artist" :class="$style.trackArtist">{{ track.artist }}</span>
            </div>
            <span :class="$style.colDur">{{ formatDuration(track.duration_secs) }}</span>
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
    </NSpin>
  </div>
</template>

<style module>
.container {
  padding: 24px;
  max-width: 960px;
}

.toolbar {
  margin-bottom: 24px;
}

.content {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

/* Header: cover + info */
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

.coverImg {
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
  background: linear-gradient(135deg, #1a2332, #0f1419);
  font-size: 72px;
  opacity: 0.3;
}

.albumInfo {
  flex: 1;
  min-width: 0;
  padding-bottom: 4px;
}

.albumLabel {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  opacity: 0.5;
  margin-bottom: 8px;
}

.albumName {
  font-size: 32px;
  font-weight: 800;
  line-height: 1.1;
  margin: 0 0 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.albumMeta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  opacity: 0.7;
  flex-wrap: wrap;
  margin-bottom: 20px;
}

.artistName {
  font-weight: 600;
  opacity: 1;
}

.metaSep {
  opacity: 0.4;
}

.actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

/* Track list */
.trackList {
  border-top: 1px solid var(--app-border);
}

.trackListHeader {
  display: grid;
  grid-template-columns: 36px 1fr 64px 36px;
  padding: 8px 12px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  opacity: 0.4;
  border-bottom: 1px solid var(--app-border);
  user-select: none;
}

.trackRow {
  display: grid;
  grid-template-columns: 36px 1fr 64px 36px;
  align-items: center;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.trackRow:hover {
  background: var(--app-hover);
}

.trackRowActive {
  color: var(--n-primary-color, #0066cc);
}

.trackRowActive .trackTitle {
  font-weight: 600;
}

.colNum {
  text-align: right;
  opacity: 0.4;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  padding-right: 16px;
}

.colTitle {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  overflow: hidden;
}

.trackTitle {
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.trackArtist {
  font-size: 12px;
  opacity: 0.5;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.colDur {
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

.trackRow:hover .addBtn {
  opacity: 0.5;
}

.addBtn:hover {
  opacity: 1 !important;
  background: rgba(128, 128, 128, 0.15);
}

/* Responsive: stack vertically on narrow screens */
@media (max-width: 600px) {
  .header {
    flex-direction: column;
    align-items: flex-start;
  }

  .coverWrapper {
    width: 160px;
    height: 160px;
  }

  .albumName {
    font-size: 24px;
  }
}
</style>
