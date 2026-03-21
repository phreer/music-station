<script setup lang="ts">
import { ref, computed } from 'vue'
import { NCard, NButton, NTabs, NTab } from 'naive-ui'
import { Play, ListPlus, ChevronDown, Music, Disc3 } from 'lucide-vue-next'
import type { Artist } from '@/types'
import { coverUrl } from '@/api/client'
import { formatDuration, formatDurationLong } from '@/utils/format'
import { usePlayerStore } from '@/stores/player'
import { useQueueStore } from '@/stores/queue'
import { useLibraryStore } from '@/stores/library'

const props = defineProps<{ artist: Artist }>()

const player = usePlayerStore()
const queue = useQueueStore()
const library = useLibraryStore()

const expanded = ref(false)
const activeTab = ref<'albums' | 'tracks'>('albums')

const artistTracks = computed(() =>
  library.allTracks.filter((t) => t.artist === props.artist.name || t.album_artist === props.artist.name),
)

const coverTrack = computed(() =>
  artistTracks.value.find((t) => t.has_cover),
)

function playArtist() {
  const ids = artistTracks.value.map((t) => t.id)
  if (!ids.length) return
  queue.setQueue(ids, 0)
  player.playTrack(ids[0]!)
}

function playAlbum(albumName: string) {
  const tracks = artistTracks.value.filter((t) => t.album === albumName)
  const ids = tracks.map((t) => t.id)
  if (!ids.length) return
  queue.setQueue(ids, 0)
  player.playTrack(ids[0]!)
}
</script>

<template>
  <NCard :class="$style.card" hoverable>
    <div :class="$style.header" @click="expanded = !expanded">
      <div :class="$style.avatar">
        <img
          v-if="coverTrack"
          :src="coverUrl(coverTrack.id)"
          :class="$style.avatarImg"
          loading="lazy"
        />
        <div v-else :class="$style.avatarPlaceholder">
          <Music :size="28" />
        </div>
      </div>
      <div :class="$style.info">
        <div :class="$style.name">{{ artist.name }}</div>
        <div :class="$style.meta">
          {{ artist.album_count }} albums · {{ artist.track_count }} tracks
        </div>
      </div>
      <div :class="$style.actions">
        <NButton circle secondary size="small" @click.stop="playArtist">
          <template #icon><Play :size="14" /></template>
        </NButton>
        <NButton circle quaternary size="small" @click.stop="queue.addMultiple(artistTracks.map(t => t.id))">
          <template #icon><ListPlus :size="14" /></template>
        </NButton>
        <ChevronDown
          :size="16"
          :class="[$style.chevron, expanded && $style.chevronOpen]"
        />
      </div>
    </div>

    <Transition name="expand">
      <div v-if="expanded" :class="$style.content">
        <NTabs v-model:value="activeTab" size="small" :class="$style.tabs">
          <NTab name="albums">
            <template #default>
              <span><Disc3 :size="12" style="margin-right:4px;vertical-align:middle" />Albums</span>
            </template>
          </NTab>
          <NTab name="tracks">
            <template #default>
              <span><Music :size="12" style="margin-right:4px;vertical-align:middle" />Tracks</span>
            </template>
          </NTab>
        </NTabs>

        <!-- Albums tab -->
        <div v-if="activeTab === 'albums'">
          <div
            v-for="album in artist.albums"
            :key="album.name"
            :class="$style.albumRow"
          >
            <div :class="$style.albumInfo">
              <div :class="$style.albumName">{{ album.name }}</div>
              <div :class="$style.albumMeta">{{ album.track_count }} tracks · {{ formatDurationLong(album.total_duration_secs) }}</div>
            </div>
            <NButton circle quaternary size="tiny" @click="playAlbum(album.name)">
              <template #icon><Play :size="12" /></template>
            </NButton>
          </div>
        </div>

        <!-- Tracks tab -->
        <div v-if="activeTab === 'tracks'">
          <div
            v-for="track in artistTracks"
            :key="track.id"
            :class="[$style.trackRow, track.id === player.currentTrackId && $style.trackRowActive]"
            @click="() => { queue.addToQueue(track.id); player.playTrack(track.id) }"
          >
            <span :class="$style.trackTitle">{{ track.title }}</span>
            <span :class="$style.trackAlbum">{{ track.album }}</span>
            <span :class="$style.trackDur">{{ formatDuration(track.duration_secs) }}</span>
          </div>
        </div>
      </div>
    </Transition>
  </NCard>
</template>

<style module>
.card { overflow: hidden; }

.header { display: flex; align-items: center; gap: 12px; cursor: pointer; padding: 4px 0; }

.avatar { width: 56px; height: 56px; border-radius: 50%; overflow: hidden; flex-shrink: 0; }
.avatarImg { width: 100%; height: 100%; object-fit: cover; }
.avatarPlaceholder {
  width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;
  background: linear-gradient(135deg, #1a2332, #0f1419); opacity: 0.5;
}

.info { flex: 1; min-width: 0; }
.name { font-weight: 600; font-size: 15px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.meta { font-size: 12px; opacity: 0.6; margin-top: 2px; }

.actions { display: flex; align-items: center; gap: 4px; flex-shrink: 0; }
.chevron { transition: transform 0.2s; opacity: 0.4; }
.chevronOpen { transform: rotate(180deg); }

.content { border-top: 1px solid var(--n-border-color, #e0e0e0); margin-top: 12px; padding-top: 8px; }
.tabs { margin-bottom: 8px; }

.albumRow { display: flex; align-items: center; gap: 8px; padding: 6px 4px; border-radius: 4px; }
.albumRow:hover { background: var(--n-merged-color, rgba(0,0,0,0.04)); }
.albumInfo { flex: 1; min-width: 0; }
.albumName { font-size: 13px; font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.albumMeta { font-size: 11px; opacity: 0.5; }

.trackRow { display: flex; align-items: center; gap: 8px; padding: 5px 4px; border-radius: 4px; cursor: pointer; font-size: 13px; }
.trackRow:hover { background: var(--n-merged-color, rgba(0,0,0,0.04)); }
.trackRowActive { color: var(--n-primary-color, #0066cc); font-weight: 600; }
.trackTitle { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.trackAlbum { flex: 0.8; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; opacity: 0.5; font-size: 11px; }
.trackDur { opacity: 0.5; font-size: 11px; font-variant-numeric: tabular-nums; flex-shrink: 0; }
</style>
