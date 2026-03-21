<script setup lang="ts">
import { NDataTable, NButton, type DataTableColumns } from 'naive-ui'
import { h, computed, ref } from 'vue'
import { Play, ListPlus, ListMusic } from 'lucide-vue-next'
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

function handlePlay(track: Track) {
  // Build queue from current track list, start at this track
  const ids = props.tracks.map((t) => t.id)
  const idx = ids.indexOf(track.id)
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

const columns = computed<DataTableColumns<Track>>(() => [
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
            background: 'var(--n-merged-color, #f0f0f0)',
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
    ellipsis: { tooltip: true },
    render(row) {
      return h('div', {}, [
        h(
          'div',
          {
            style: {
              fontWeight: row.id === player.currentTrackId ? '600' : '400',
              color: row.id === player.currentTrackId ? 'var(--n-primary-color)' : undefined,
            },
          },
          row.title || 'Unknown Title',
        ),
        h(
          'div',
          { style: { fontSize: '12px', opacity: 0.6 } },
          row.artist || 'Unknown Artist',
        ),
      ])
    },
  },
  {
    key: 'album',
    title: 'Album',
    width: 200,
    ellipsis: { tooltip: true },
    render(row) {
      return row.album || '-'
    },
  },
  {
    key: 'duration',
    title: 'Duration',
    width: 80,
    align: 'right',
    render(row) {
      return formatDuration(row.duration)
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
        return h('div', { style: { display: 'flex', gap: '4px' } }, [
          h(
            NButton,
            {
              quaternary: true,
              circle: true,
              size: 'small',
              onClick: (e: Event) => {
                e.stopPropagation()
                handlePlay(row)
              },
            },
            { icon: () => h(Play, { size: 14 }) },
          ),
          h(
            NButton,
            {
              quaternary: true,
              circle: true,
              size: 'small',
              onClick: (e: Event) => {
                e.stopPropagation()
                handleAddToQueue(row)
              },
            },
            { icon: () => h(ListPlus, { size: 14 }) },
          ),
          h(
            NButton,
            {
              quaternary: true,
              circle: true,
              size: 'small',
              title: 'Add to playlist',
              onClick: (e: Event) => {
                e.stopPropagation()
                handleAddToPlaylist(row)
              },
            },
            { icon: () => h(ListMusic, { size: 14 }) },
          ),
        ])
      },
    },
  ])

function handleRowClick(row: Track) {
  handlePlay(row)
}

const rowProps = (row: Track) => ({
  style: {
    cursor: 'pointer',
  },
  onClick: () => handleRowClick(row),
})
</script>

<template>
  <div>
    <NDataTable
      :columns="columns"
      :data="tracks"
      :row-props="rowProps"
      :max-height="'calc(100vh - 250px)'"
      virtual-scroll
      size="small"
      striped
    />
    <AddToPlaylistModal v-model:show="showAddToPlaylist" :track="addToPlaylistTrack" />
  </div>
</template>
