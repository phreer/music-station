<script setup lang="ts">
import { NDataTable, type DataTableColumns, type DataTableRowKey } from 'naive-ui'
import { computed, h, onBeforeUnmount, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import type { Track } from '@/types'
import { coverUrl } from '@/api/client'
import { formatDuration } from '@/utils/format'
import { usePlayerStore } from '@/stores/player'
import { useQueueStore } from '@/stores/queue'
import AddToPlaylistModal from '@/components/modals/AddToPlaylistModal.vue'
import TrackFavoriteButton from '@/components/tracks/TrackFavoriteButton.vue'
import { useResizableTrackColumns } from '@/composables/useResizableTrackColumns'

const { tracks } = defineProps<{
  tracks: Track[]
}>()

const player = usePlayerStore()
const queue = useQueueStore()
const router = useRouter()

const showAddToPlaylist = ref(false)
const addToPlaylistTrack = ref<Track | null>(null)
const scrollbarProps = { trigger: 'none' as const }
const fixedColumnWidth = 50 + 162
const tableWrapper = ref<HTMLElement | null>(null)
const containerWidth = ref(0)
let resizeObserver: ResizeObserver | null = null
const { getWidth, startResize } = useResizableTrackColumns(
  'track-list-column-widths',
  [
    { key: 'title', defaultWidth: 320, minWidth: 180, maxWidth: 640 },
    { key: 'album', defaultWidth: 220, minWidth: 120, maxWidth: 480 },
    { key: 'duration', defaultWidth: 104, minWidth: 96, maxWidth: 180 },
    { key: 'play_count', defaultWidth: 72, minWidth: 56, maxWidth: 140 },
  ],
)

const tableContentWidth = computed(() =>
  fixedColumnWidth
  + getWidth('title')
  + getWidth('album')
  + getWidth('duration')
  + getWidth('play_count'),
)

const tableWidth = computed(() => Math.max(tableContentWidth.value, containerWidth.value))
const tableScrollX = computed(() => (
  tableContentWidth.value > containerWidth.value ? tableContentWidth.value : undefined
))
const spacerWidth = computed(() => tableWidth.value - tableContentWidth.value)

function lockedWidth(key: string): { width: number, minWidth: number, maxWidth: number } {
  const width = getWidth(key)
  return { width, minWidth: width, maxWidth: width }
}

function fixedWidth(width: number): { width: number, minWidth: number, maxWidth: number } {
  return { width, minWidth: width, maxWidth: width }
}

function resizableTitle(label: string, key: string, align: 'left' | 'right' = 'left') {
  return h('div', { class: ['track-resize-header', align === 'right' && 'track-resize-header-right'] }, [
    h('span', { class: 'track-resize-header-label' }, label),
    h('span', {
      class: 'track-column-resize-handle',
      onMousedown: (event: MouseEvent) => {
        event.preventDefault()
        event.stopPropagation()
        startResize(key, event)
      },
    }),
  ])
}

function handlePlay(track: Track) {
  // Add track to queue (if absent) and update currentIndex before playing,
  // so the queue panel highlights the correct track immediately.
  queue.playInQueue(track.id)
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
const columns = computed<DataTableColumns<Track>>(() => [
  {
    key: 'cover',
    title: '',
    ...fixedWidth(50),
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
    title: () => resizableTitle('Title', 'title'),
    ...lockedWidth('title'),
    render(row) {
      const artistEl = row.artist
        ? h('span', {
            class: 'track-artist-text track-nav-link',
            onClick: (e: Event) => {
              e.stopPropagation()
              router.push({ name: 'artist-detail', params: { name: row.artist } })
            },
          }, row.artist)
        : h('span', { class: 'track-artist-text' }, 'Unknown Artist')
      return h('div', { class: 'track-title-cell' }, [
        h('div', { class: 'track-title-text' }, row.title || 'Unknown Title'),
        artistEl,
      ])
    },
  },
  {
    key: 'album',
    title: () => resizableTitle('Album', 'album'),
    ...lockedWidth('album'),
    render(row) {
      if (!row.album) return h('span', { class: 'track-album-text' }, '-')
      return h('span', {
        class: 'track-album-text track-nav-link',
        onClick: (e: Event) => {
          e.stopPropagation()
          router.push({ name: 'album-detail', params: { name: row.album } })
        },
      }, row.album)
    },
  },
  {
    key: 'duration',
    title: () => resizableTitle('Duration', 'duration', 'right'),
    ...lockedWidth('duration'),
    align: 'right',
    render(row) {
      return formatDuration(row.duration_secs)
    },
  },
  {
    key: 'play_count',
    title: () => resizableTitle('Plays', 'play_count', 'right'),
    ...lockedWidth('play_count'),
    align: 'right',
  },
  {
    key: 'spacer',
    title: '',
    ...fixedWidth(spacerWidth.value),
  },
  {
    key: 'actions',
    title: '',
    ...fixedWidth(162),
    render(row) {
      return h('div', { class: 'track-actions' }, [
        h(TrackFavoriteButton, {
          trackId: row.id,
          size: 28,
          iconSize: 14,
        }),
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
])

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

onMounted(() => {
  const updateContainerWidth = () => {
    containerWidth.value = tableWrapper.value?.clientWidth ?? 0
  }

  updateContainerWidth()
  if (!tableWrapper.value) return
  resizeObserver = new ResizeObserver(updateContainerWidth)
  resizeObserver.observe(tableWrapper.value)
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
})

</script>

<template>
  <div
    ref="tableWrapper"
    class="track-list-wrapper"
    :class="{ 'track-list-wrapper--scrollable': tableScrollX !== undefined }"
  >
    <NDataTable
      :columns="columns"
      :data="tracks"
      :row-key="rowKey"
      :row-props="rowProps"
      :row-class-name="rowClassName"
      :scroll-x="tableScrollX"
      :scrollbar-props="scrollbarProps"
      table-layout="fixed"
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

.track-list-wrapper--scrollable .n-data-table-base-table-body {
  /* The scrollbar lives in this gutter instead of overlapping a playable row. */
  padding-bottom: 16px;
}

.track-list-wrapper .n-scrollbar {
  --n-scrollbar-width: 14px !important;
  --n-scrollbar-color: var(--app-scrollbar-thumb) !important;
  --n-scrollbar-color-hover: var(--app-scrollbar-thumb-hover) !important;
  --n-scrollbar-rail-color: transparent !important;
}

.track-list-wrapper--scrollable .n-scrollbar {
  --n-scrollbar-height: 10px !important;
}

.track-list-wrapper .n-scrollbar-rail--vertical:has(> .n-scrollbar-rail__scrollbar) {
  pointer-events: auto;
  border-radius: 999px;
  background: var(--app-scrollbar-track);
}

.track-list-wrapper .n-scrollbar-rail--vertical > .n-scrollbar-rail__scrollbar {
  border: 3px solid transparent;
  background-clip: padding-box;
}

.track-list-wrapper--scrollable .n-scrollbar-rail--horizontal:has(> .n-scrollbar-rail__scrollbar) {
  pointer-events: auto;
  border-radius: 999px;
  background: var(--app-scrollbar-track);
}

.track-resize-header {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
  min-width: 0;
}

.track-resize-header-right {
  justify-content: flex-end;
}

.track-resize-header-right .track-resize-header-label {
  padding-right: 14px;
}

.track-resize-header-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.track-column-resize-handle {
  position: absolute;
  top: -10px;
  right: -18px;
  width: 28px;
  height: calc(100% + 20px);
  cursor: col-resize;
  z-index: 1;
}

.track-column-resize-handle::after {
  content: '';
  position: absolute;
  top: 10px;
  bottom: 10px;
  left: 11px;
  width: 2px;
  border-radius: 999px;
  background: var(--app-border);
  opacity: 0;
  transition: opacity 0.15s, background 0.15s;
}

.track-resize-header:hover .track-column-resize-handle::after,
.track-column-resize-handle:hover::after {
  opacity: 0.7;
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

/* Clickable artist/album navigation links */
.track-nav-link {
  cursor: pointer;
}

.track-nav-link:hover {
  text-decoration: underline;
  opacity: 0.9;
}

/* Lightweight native action buttons — replaces NButton + Lucide component instances */
.track-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  padding-right: 18px;
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
