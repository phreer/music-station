<script setup lang="ts">
import { ref, computed } from 'vue'
import { NCard, NTabs, NTab } from 'naive-ui'
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
  library.getTracksByArtist(props.artist.name),
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
          <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18V5l12-2v13"></path><circle cx="6" cy="18" r="3"></circle><circle cx="18" cy="16" r="3"></circle></svg>
        </div>
      </div>
      <div :class="$style.info">
        <div :class="$style.name">{{ artist.name }}</div>
        <div :class="$style.meta">
          {{ artist.album_count }} albums · {{ artist.track_count }} tracks
        </div>
      </div>
      <div :class="$style.actions">
        <button :class="$style.actionBtn" title="Play all" @click.stop="playArtist">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="currentColor" stroke="none"><polygon points="6 3 20 12 6 21 6 3"></polygon></svg>
        </button>
        <button :class="$style.actionBtn" title="Add all to queue" @click.stop="queue.addMultiple(artistTracks.map(t => t.id))">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 12H3"></path><path d="M16 6H3"></path><path d="M16 18H3"></path><path d="M18 9v6"></path><path d="M21 12h-6"></path></svg>
        </button>
        <svg :class="[$style.chevron, expanded && $style.chevronOpen]" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"></path></svg>
      </div>
    </div>

    <Transition name="expand">
      <div v-if="expanded" :class="$style.content">
        <NTabs v-model:value="activeTab" size="small" :class="$style.tabs">
          <NTab name="albums">
            <template #default>
              <span>
                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="margin-right:4px;vertical-align:middle"><line x1="12" x2="12" y1="2" y2="22"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path><circle cx="12" cy="12" r="10"></circle></svg>Albums
              </span>
            </template>
          </NTab>
          <NTab name="tracks">
            <template #default>
              <span>
                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="margin-right:4px;vertical-align:middle"><path d="M9 18V5l12-2v13"></path><circle cx="6" cy="18" r="3"></circle><circle cx="18" cy="16" r="3"></circle></svg>Tracks
              </span>
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
            <button :class="$style.albumPlayBtn" title="Play album" @click="playAlbum(album.name)">
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="currentColor" stroke="none"><polygon points="6 3 20 12 6 21 6 3"></polygon></svg>
            </button>
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

.actionBtn {
  display: inline-flex; align-items: center; justify-content: center;
  width: 28px; height: 28px; border-radius: 50%;
  border: none; cursor: pointer; padding: 0;
  background: transparent; color: inherit;
  opacity: 0.6; transition: opacity 0.15s, background 0.15s;
}
.actionBtn:hover { opacity: 1; background: rgba(128,128,128,0.12); }

.chevron { transition: transform 0.2s; opacity: 0.4; }
.chevronOpen { transform: rotate(180deg); }

.content { border-top: 1px solid var(--app-border); margin-top: 12px; padding-top: 8px; }
.tabs { margin-bottom: 8px; }

.albumRow { display: flex; align-items: center; gap: 8px; padding: 6px 4px; border-radius: 4px; }
.albumRow:hover { background: var(--app-hover); }
.albumInfo { flex: 1; min-width: 0; }
.albumName { font-size: 13px; font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.albumMeta { font-size: 11px; opacity: 0.5; }

.albumPlayBtn {
  flex-shrink: 0; display: inline-flex; align-items: center; justify-content: center;
  width: 24px; height: 24px; border: none; border-radius: 50%;
  background: transparent; color: inherit; cursor: pointer; padding: 0;
  opacity: 0; transition: opacity 0.15s, background 0.15s;
}
.albumRow:hover .albumPlayBtn { opacity: 0.5; }
.albumPlayBtn:hover { opacity: 1 !important; background: rgba(128,128,128,0.15); }

.trackRow { display: flex; align-items: center; gap: 8px; padding: 5px 4px; border-radius: 4px; cursor: pointer; font-size: 13px; }
.trackRow:hover { background: var(--app-hover); }
.trackRowActive { color: var(--n-primary-color, #0066cc); font-weight: 600; }
.trackTitle { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.trackAlbum { flex: 0.8; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; opacity: 0.5; font-size: 11px; }
.trackDur { opacity: 0.5; font-size: 11px; font-variant-numeric: tabular-nums; flex-shrink: 0; }
</style>
