<script setup lang="ts">
import { ref, computed } from 'vue'
import { NCard, NButton, NPopconfirm } from 'naive-ui'
import { Play, ListPlus, Trash2, ChevronDown } from 'lucide-vue-next'
import type { Playlist } from '@/types'
import { coverUrl } from '@/api/client'
import { formatDuration } from '@/utils/format'
import { usePlayerStore } from '@/stores/player'
import { useQueueStore } from '@/stores/queue'
import { usePlaylistStore } from '@/stores/playlists'
import { useLibraryStore } from '@/stores/library'

const props = defineProps<{ playlist: Playlist }>()

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

async function handleDelete() {
  await playlistStore.deletePlaylist(props.playlist.id)
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
        <NButton circle type="primary" @click.stop="playPlaylist">
          <template #icon><Play :size="18" /></template>
        </NButton>
        <NButton circle secondary @click.stop="queue.addMultiple(playlist.tracks)">
          <template #icon><ListPlus :size="18" /></template>
        </NButton>
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
      <NPopconfirm @positive-click="handleDelete">
        <template #trigger>
          <NButton quaternary size="tiny" :class="$style.deleteBtn">
            <template #icon><Trash2 :size="12" /></template>
            Delete
          </NButton>
        </template>
        Delete playlist "{{ playlist.name }}"?
      </NPopconfirm>
      <div :class="$style.expandToggle" @click="expanded = !expanded">
        <ChevronDown :size="16" :class="[$style.chevron, expanded && $style.chevronOpen]" />
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
          <NButton quaternary circle size="tiny" @click.stop="handleRemoveTrack(track.id)">
            <template #icon><Trash2 :size="10" /></template>
          </NButton>
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

.info { padding: 0 2px; }
.playlistName { font-weight: 600; font-size: 14px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.playlistMeta { font-size: 12px; opacity: 0.6; margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.cardFooter { display: flex; align-items: center; justify-content: space-between; padding-top: 8px; }
.deleteBtn { opacity: 0.5; }
.deleteBtn:hover { opacity: 1; }
.expandToggle { cursor: pointer; opacity: 0.4; padding: 4px; }
.expandToggle:hover { opacity: 0.8; }
.chevron { transition: transform 0.2s; }
.chevronOpen { transform: rotate(180deg); }

.trackList { border-top: 1px solid var(--n-border-color, #e0e0e0); margin-top: 8px; }
.emptyTracks { padding: 12px 4px; font-size: 13px; opacity: 0.4; }
.trackRow {
  display: flex; align-items: center; gap: 8px; padding: 6px 4px;
  cursor: pointer; border-radius: 4px; font-size: 13px; transition: background 0.15s;
}
.trackRow:hover { background: var(--n-merged-color, rgba(0,0,0,0.04)); }
.trackRowActive { color: var(--n-primary-color, #0066cc); font-weight: 600; }
.trackTitle { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.trackArtist { flex: 0.8; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; opacity: 0.5; font-size: 11px; }
.trackDur { opacity: 0.5; font-size: 11px; font-variant-numeric: tabular-nums; flex-shrink: 0; }
</style>
