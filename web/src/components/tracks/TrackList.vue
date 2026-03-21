<script setup lang="ts">
import { NDataTable, type DataTableColumns, type DataTableRowKey } from 'naive-ui'
import { h, computed, ref } from 'vue'
import type { Track } from '@/types'
import { coverUrl } from '@/api/client'
import { formatDuration } from '@/utils/format'
import { usePlayerStore } from '@/stores/player'
import { useQueueStore } from '@/stores/queue'
import AddToPlaylistModal from '@/components/modals/AddToPlaylistModal.vue'

const props = defineProps<{
  tracks: Track[]
}>()

const player = usePlayerStore()
const queue = useQueueStore()

const showAddToPlaylist = ref(false)
const addToPlaylistTrack = ref<Track | null>(null)

// Cached ID-to-index map — avoids O(n) indexOf on every play click
const trackIdIndexMap = computed(() => {
  const map = new Map<string, number>()
  for (let i = 0; i < props.tracks.length; i++) {
    map.set(props.tracks[i]!.id, i)
  }
  return map
})

function handlePlay(track: Track) {
  const ids = props.tracks.map((t) => t.id)
  const idx = trackIdIndexMap.value.get(track.id) ?? 0
  queue.setQueue(ids, idx)
  player.playTrack(track.id)
}

function handleAddToQueue(track: Track) {
  queue.addToQueue(track.id)
}

function handleAddToPlaylist(track: Track) {
  addToPlaylistTrack.value = track
  showAddToPlaylist.value = true
}

// Inline SVG strings — avoids 6 component instances (3 NButton + 3 Lucide) per visible row
const playIcon = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="6 3 20 12 6 21 6 3"></polygon></svg>'
const addQueueIcon = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 12H3"></path><path d="M16 6H3"></path><path d="M16 18H3"></path><path d="M18 9v6"></path><path d="M21 12h-6"></path></svg>'
const playlistIcon = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15V6"></path><path d="M18.5 18a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5Z"></path><path d="M12 12H3"></path><path d="M16 6H3"></path><path d="M12 18H3"></path></svg>'

// Column definitions — NO reactive dependency on player state.
// Current track highlighting is handled via row-class-name instead.
const columns: DataTableColumns<Track> = [
  {
    key: 'cover',
    title: '',
    width: 50,
    render(row) {
      if (row.has_cover) {
        return h('img', {
          src: coverUrl(row.id),
          style: {
            width: '36px',
            height: '36px',
            borderRadius: '4px',
            objectFit: 'cover',
          },
          loading: 'lazy',
        })
      }
      return h(
        'div',
        {
          style: {
            width: '36px',
            height: '36px',
            borderRadius: '4px',
            background: 'var(--app-placeholder-bg)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: '14px',
            opacity: 0.3,
          },
        },
        '\u266A',
      )
    },
  },
  {
    key: 'title',
    title: 'Title',
    minWidth: 200,
    render(row) {
      return h('div', { class: 'track-title-cell' }, [
        h('div', { class: 'track-title-text' }, row.title || 'Unknown Title'),
        h('div', { class: 'track-artist-text' }, row.artist || 'Unknown Artist'),
      ])
    },
  },
  {
    key: 'album',
    title: 'Album',
    width: 200,
    render(row) {
      return h('span', { class: 'track-album-text' }, row.album || '-')
    },
  },
  {
    key: 'duration',
    title: 'Duration',
    width: 80,
    align: 'right',
    render(row) {
      return formatDuration(row.duration_secs)
    },
  },
  {
    key: 'play_count',
    title: 'Plays',
    width: 60,
    align: 'right',
  },
  {
    key: 'actions',
    title: '',
    width: 112,
    render(row) {
      return h('div', { class: 'track-actions' }, [
        h('button', {
          class: 'track-action-btn',
          title: 'Play',
          innerHTML: playIcon,
          onClick: (e: Event) => {
            e.stopPropagation()
            handlePlay(row)
          },
        }),
        h('button', {
          class: 'track-action-btn',
          title: 'Add to queue',
          innerHTML: addQueueIcon,
          onClick: (e: Event) => {
            e.stopPropagation()
            handleAddToQueue(row)
          },
        }),
        h('button', {
          class: 'track-action-btn',
          title: 'Add to playlist',
          innerHTML: playlistIcon,
          onClick: (e: Event) => {
            e.stopPropagation()
            handleAddToPlaylist(row)
          },
        }),
      ])
    },
  },
]

// Stable row key for efficient virtual-list DOM diffing
const rowKey = (row: Track): DataTableRowKey => row.id

// Current-track highlighting via row class — decoupled from columns definition
const rowClassName = (row: Track): string => {
  return row.id === player.currentTrackId ? 'track-row-playing' : ''
}

const rowProps = (row: Track) => ({
  style: { cursor: 'pointer' },
  onClick: () => handlePlay(row),
})
</script>

<template>
  <div>
    <NDataTable
      :columns="columns"
      :data="tracks"
      :row-key="rowKey"
      :row-props="rowProps"
      :row-class-name="rowClassName"
      :max-height="'calc(100vh - 250px)'"
      virtual-scroll
      size="small"
      striped
    />
    <AddToPlaylistModal v-model:show="showAddToPlaylist" :track="addToPlaylistTrack" />
  </div>
</template>

<style>
/* Current-track highlighting via row class (decoupled from columns computed) */
.track-row-playing .track-title-text {
  font-weight: 600;
  color: var(--n-primary-color);
}

/* CSS-based ellipsis — replaces NEllipsis + NTooltip component instances */
.track-title-text,
.track-album-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.track-artist-text {
  font-size: 12px;
  opacity: 0.6;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Lightweight native action buttons — replaces NButton + Lucide component instances */
.track-actions {
  display: flex;
  gap: 4px;
}

.track-action-btn {
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
  opacity: 0.6;
  transition: opacity 0.15s, background 0.15s;
}

.track-action-btn:hover {
  opacity: 1;
  background: rgba(128, 128, 128, 0.15);
}

.track-action-btn:active {
  background: rgba(128, 128, 128, 0.25);
}
</style>
