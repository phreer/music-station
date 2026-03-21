<script setup lang="ts">
import { ref, computed } from 'vue'
import { NCard } from 'naive-ui'
import type { Playlist } from '@/types'
import { coverUrl } from '@/api/client'
import { formatDuration } from '@/utils/format'
import { usePlayerStore } from '@/stores/player'
import { useQueueStore } from '@/stores/queue'
import { usePlaylistStore } from '@/stores/playlists'
import { useLibraryStore } from '@/stores/library'

const props = defineProps<{ playlist: Playlist }>()

const emit = defineEmits<{
  'request-delete': [playlistId: string, playlistName: string]
}>()

const player = usePlayerStore()
const queue = useQueueStore()
const playlistStore = usePlaylistStore()
const library = useLibraryStore()

const expanded = ref(false)

const tracks = computed(() =>
  props.playlist.tracks
    .map((id) => library.findTrack(id))
    .filter((t) => t !== undefined),
)

// Up to 4 cover images for the grid
const coverTrackIds = computed(() =>
  tracks.value.filter((t) => t.has_cover).slice(0, 4).map((t) => t.id),
)

const totalDuration = computed(() =>
  tracks.value.reduce((sum, t) => sum + (t.duration_secs ?? 0), 0),
)

function playPlaylist() {
  const ids = tracks.value.map((t) => t.id)
  if (!ids.length) return
  queue.setQueue(ids, 0)
  player.playTrack(ids[0]!)
}

function playTrack(trackId: string) {
  const ids = tracks.value.map((t) => t.id)
  const idx = ids.indexOf(trackId)
  queue.setQueue(ids, idx >= 0 ? idx : 0)
  player.playTrack(trackId)
}

async function handleRemoveTrack(trackId: string) {
  await playlistStore.removeTrack(props.playlist.id, trackId)
}
</script>

<template>
  <NCard :class="$style.card" hoverable>
    <div :class="$style.coverWrapper" @click="expanded = !expanded">
      <!-- Cover grid (up to 4 images) -->
      <div v-if="coverTrackIds.length > 0" :class="$style.coverGrid">
        <img
          v-for="id in coverTrackIds"
          :key="id"
          :src="coverUrl(id)"
          :class="$style.coverGridImg"
          loading="lazy"
        />
      </div>
      <div v-else :class="$style.coverPlaceholder">&#9835;</div>
      <div :class="$style.overlay">
        <button :class="[$style.iconBtn, $style.iconBtnPrimary]" @click.stop="playPlaylist" title="Play playlist">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="currentColor" stroke="none"><polygon points="6 3 20 12 6 21 6 3"></polygon></svg>
        </button>
        <button :class="$style.iconBtn" @click.stop="queue.addMultiple(playlist.tracks)" title="Add to queue">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 12H3"></path><path d="M16 6H3"></path><path d="M16 18H3"></path><path d="M18 9v6"></path><path d="M21 12h-6"></path></svg>
        </button>
      </div>
    </div>

    <div :class="$style.info">
      <div :class="$style.playlistName">{{ playlist.name }}</div>
      <div :class="$style.playlistMeta" v-if="playlist.description">{{ playlist.description }}</div>
      <div :class="$style.playlistMeta">
        {{ playlist.tracks.length }} tracks · {{ formatDuration(totalDuration) }}
      </div>
    </div>

    <div :class="$style.cardFooter">
      <button
        :class="$style.deleteBtn"
        @click="emit('request-delete', playlist.id, playlist.name)"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"></path><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path></svg>
        Delete
      </button>
      <div :class="$style.expandToggle" @click="expanded = !expanded">
        <svg :class="[$style.chevron, expanded && $style.chevronOpen]" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"></path></svg>
      </div>
    </div>

    <Transition name="expand">
      <div v-if="expanded" :class="$style.trackList">
        <div v-if="tracks.length === 0" :class="$style.emptyTracks">No tracks</div>
        <div
          v-for="track in tracks"
          :key="track.id"
          :class="[$style.trackRow, track.id === player.currentTrackId && $style.trackRowActive]"
          @click="playTrack(track.id)"
        >
          <span :class="$style.trackTitle">{{ track.title }}</span>
          <span :class="$style.trackArtist">{{ track.artist }}</span>
          <span :class="$style.trackDur">{{ formatDuration(track.duration_secs) }}</span>
          <button
            :class="$style.trackRemoveBtn"
            title="Remove from playlist"
            @click.stop="handleRemoveTrack(track.id)"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"></path><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path></svg>
          </button>
        </div>
      </div>
    </Transition>
  </NCard>
</template>

<style module>
.card { overflow: hidden; }

.coverWrapper {
  position: relative; aspect-ratio: 1; overflow: hidden; cursor: pointer;
  border-radius: 6px; margin-bottom: 10px;
}
.coverGrid {
  width: 100%; height: 100%; display: grid;
  grid-template-columns: 1fr 1fr; grid-template-rows: 1fr 1fr; gap: 2px;
}
.coverGridImg { width: 100%; height: 100%; object-fit: cover; }
.coverPlaceholder {
  width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;
  background: linear-gradient(135deg, #1a2332, #2a3f55); font-size: 48px; opacity: 0.3;
}
.overlay {
  position: absolute; inset: 0; display: flex; align-items: center; justify-content: center;
  gap: 12px; background: rgba(0,0,0,0.5); opacity: 0; transition: opacity 0.2s;
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
.playlistName { font-weight: 600; font-size: 14px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.playlistMeta { font-size: 12px; opacity: 0.6; margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.cardFooter { display: flex; align-items: center; justify-content: space-between; padding-top: 8px; }

.deleteBtn {
  display: inline-flex; align-items: center; gap: 4px;
  border: none; background: transparent; color: inherit; cursor: pointer;
  font-size: 12px; padding: 4px 8px; border-radius: 4px;
  opacity: 0.4; transition: opacity 0.15s, background 0.15s, color 0.15s;
}
.deleteBtn:hover { opacity: 1; background: var(--app-danger-bg); color: var(--app-danger); }

.expandToggle { cursor: pointer; opacity: 0.4; padding: 4px; }
.expandToggle:hover { opacity: 0.8; }
.chevron { transition: transform 0.2s; }
.chevronOpen { transform: rotate(180deg); }

.trackList { border-top: 1px solid var(--app-border); margin-top: 8px; }
.emptyTracks { padding: 12px 4px; font-size: 13px; opacity: 0.4; }
.trackRow {
  display: flex; align-items: center; gap: 8px; padding: 6px 4px;
  cursor: pointer; border-radius: 4px; font-size: 13px; transition: background 0.15s;
}
.trackRow:hover { background: var(--app-hover); }
.trackRowActive { color: var(--n-primary-color, #0066cc); font-weight: 600; }
.trackTitle { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.trackArtist { flex: 0.8; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; opacity: 0.5; font-size: 11px; }
.trackDur { opacity: 0.5; font-size: 11px; font-variant-numeric: tabular-nums; flex-shrink: 0; }

.trackRemoveBtn {
  flex-shrink: 0; display: inline-flex; align-items: center; justify-content: center;
  width: 22px; height: 22px; border: none; border-radius: 50%;
  background: transparent; color: inherit; cursor: pointer; padding: 0;
  opacity: 0; transition: opacity 0.15s, background 0.15s;
}
.trackRow:hover .trackRemoveBtn { opacity: 0.5; }
.trackRemoveBtn:hover { opacity: 1 !important; background: var(--app-danger-bg); color: var(--app-danger); }
</style>
