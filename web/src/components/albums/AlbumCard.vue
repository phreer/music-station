<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { NCard } from 'naive-ui'
import type { Album, Track } from '@/types'
import { coverUrl } from '@/api/client'
import { formatDuration, formatDurationLong } from '@/utils/format'
import { usePlayerStore } from '@/stores/player'
import { useQueueStore } from '@/stores/queue'
import { useLibraryStore } from '@/stores/library'

const props = defineProps<{ album: Album }>()

const router = useRouter()
const player = usePlayerStore()
const queue = useQueueStore()
const library = useLibraryStore()

const expanded = ref(false)

// Use tracks from album directly (already populated by /albums endpoint)
const tracks = computed<Track[]>(() => {
  if (props.album.tracks && props.album.tracks.length > 0) return props.album.tracks
  // Fallback: use pre-computed index instead of scanning allTracks
  return library.getTracksByAlbum(props.album.name, props.album.artist)
})

const coverTrack = computed(() => tracks.value.find((t) => t.has_cover))

function playAlbum() {
  const ids = tracks.value.map((t) => t.id)
  if (ids.length === 0) return
  queue.setQueue(ids, 0)
  player.playTrack(ids[0]!)
}

function addAlbumToQueue() {
  queue.addMultiple(tracks.value.map((t) => t.id))
}

function playTrack(track: Track) {
  const ids = tracks.value.map((t) => t.id)
  const idx = ids.indexOf(track.id)
  queue.setQueue(ids, idx)
  player.playTrack(track.id)
}
</script>

<template>
  <NCard :class="$style.card" hoverable>
    <div :class="$style.coverWrapper" @click="expanded = !expanded">
      <img
        v-if="coverTrack"
        :src="coverUrl(coverTrack.id)"
        :class="$style.coverImg"
        loading="lazy"
      />
      <div v-else :class="$style.coverPlaceholder">&#9834;</div>
      <div :class="$style.overlay">
        <button :class="[$style.iconBtn, $style.iconBtnPrimary]" @click.stop="playAlbum" title="Play album">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="currentColor" stroke="none"><polygon points="6 3 20 12 6 21 6 3"></polygon></svg>
        </button>
        <button :class="$style.iconBtn" @click.stop="addAlbumToQueue" title="Add to queue">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 12H3"></path><path d="M16 6H3"></path><path d="M16 18H3"></path><path d="M18 9v6"></path><path d="M21 12h-6"></path></svg>
        </button>
      </div>
    </div>
    <div :class="$style.info">
      <div
        :class="[$style.albumName, $style.albumNameLink]"
        :title="album.name"
        @click.stop="router.push({ name: 'album-detail', params: { name: album.name } })"
      >{{ album.name }}</div>
      <div :class="$style.albumMeta">{{ album.artist }}</div>
      <div :class="$style.albumMeta">
        {{ album.track_count }} tracks · {{ formatDurationLong(album.total_duration_secs) }}
      </div>
    </div>

    <div :class="$style.expandToggle" @click="expanded = !expanded">
      <svg :class="[$style.chevron, expanded && $style.chevronOpen]" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"></path></svg>
    </div>

    <Transition name="expand">
      <div v-if="expanded" :class="$style.trackList">
        <div
          v-for="track in tracks"
          :key="track.id"
          :class="[$style.trackRow, track.id === player.currentTrackId && $style.trackRowActive]"
          @click="playTrack(track)"
        >
          <span :class="$style.trackNum">{{ track.track_number ?? '—' }}</span>
          <span :class="$style.trackTitle">{{ track.title }}</span>
          <span :class="$style.trackDur">{{ formatDuration(track.duration_secs) }}</span>
          <button
            :class="$style.trackAddBtn"
            title="Add to queue"
            @click.stop="queue.addToQueue(track.id)"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 12H3"></path><path d="M16 6H3"></path><path d="M16 18H3"></path><path d="M18 9v6"></path><path d="M21 12h-6"></path></svg>
          </button>
        </div>
      </div>
    </Transition>
  </NCard>
</template>

<style module>
.card { overflow: hidden; }

.coverWrapper {
  position: relative;
  aspect-ratio: 1;
  overflow: hidden;
  cursor: pointer;
  border-radius: 6px;
  margin-bottom: 10px;
}
.coverImg { width: 100%; height: 100%; object-fit: cover; display: block; }
.coverPlaceholder {
  width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;
  background: linear-gradient(135deg, #1a2332, #0f1419);
  font-size: 48px; opacity: 0.3;
}
.overlay {
  position: absolute; inset: 0;
  display: flex; align-items: center; justify-content: center; gap: 12px;
  background: rgba(0,0,0,0.5);
  opacity: 0; transition: opacity 0.2s;
}
.coverWrapper:hover .overlay { opacity: 1; }

.iconBtn {
  display: inline-flex; align-items: center; justify-content: center;
  width: 40px; height: 40px; border-radius: 50%;
  border: none; cursor: pointer; padding: 0;
  background: rgba(255,255,255,0.15); color: #fff;
  transition: background 0.15s, transform 0.1s;
}
.iconBtn:hover { background: rgba(255,255,255,0.3); transform: scale(1.08); }
.iconBtnPrimary { background: var(--n-primary-color, #0066cc); }
.iconBtnPrimary:hover { background: var(--n-primary-color-hover, #0077ee); }

.info { padding: 0 2px; }
.albumName { font-weight: 600; font-size: 14px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.albumNameLink { cursor: pointer; }
.albumNameLink:hover { text-decoration: underline; opacity: 0.8; }
.albumMeta { font-size: 12px; opacity: 0.6; margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.expandToggle {
  display: flex; justify-content: center; padding: 6px 0 0;
  cursor: pointer; opacity: 0.4;
}
.expandToggle:hover { opacity: 0.8; }
.chevron { transition: transform 0.2s; }
.chevronOpen { transform: rotate(180deg); }

.trackList { border-top: 1px solid var(--app-border); margin-top: 8px; }
.trackRow {
  display: flex; align-items: center; gap: 8px;
  padding: 6px 4px; cursor: pointer; border-radius: 4px;
  font-size: 13px; transition: background 0.15s;
}
.trackRow:hover { background: var(--app-hover); }
.trackRowActive { color: var(--n-primary-color, #0066cc); font-weight: 600; }
.trackNum { width: 20px; text-align: right; opacity: 0.4; font-size: 11px; flex-shrink: 0; }
.trackTitle { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.trackDur { opacity: 0.5; font-size: 11px; font-variant-numeric: tabular-nums; flex-shrink: 0; }

.trackAddBtn {
  flex-shrink: 0; display: inline-flex; align-items: center; justify-content: center;
  width: 22px; height: 22px; border: none; border-radius: 50%;
  background: transparent; color: inherit; cursor: pointer; padding: 0;
  opacity: 0; transition: opacity 0.15s, background 0.15s;
}
.trackRow:hover .trackAddBtn { opacity: 0.5; }
.trackAddBtn:hover { opacity: 1 !important; background: rgba(128,128,128,0.15); }
</style>

<style>
.expand-enter-active, .expand-leave-active { transition: all 0.2s ease; overflow: hidden; }
.expand-enter-from, .expand-leave-to { opacity: 0; max-height: 0; }
.expand-enter-to, .expand-leave-from { opacity: 1; max-height: 800px; }
</style>
