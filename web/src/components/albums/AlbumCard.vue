<script setup lang="ts">
import { tr } from '@/i18n'
import { ref, computed, watch } from 'vue'
import type { Album, Track } from '@/types'
import { coverUrl } from '@/api/client'
import { formatDuration, formatDurationLong } from '@/utils/format'
import { usePlayerStore } from '@/stores/player'
import { useQueueStore } from '@/stores/queue'
import { useLibraryStore } from '@/stores/library'

const props = defineProps<{ album: Album }>()

const player = usePlayerStore()
const queue = useQueueStore()
const library = useLibraryStore()

const expanded = ref(false)
const coverFailed = ref(false)

// Use tracks from album directly (already populated by /albums endpoint)
const tracks = computed<Track[]>(() => {
  const raw = props.album.tracks && props.album.tracks.length > 0
    ? props.album.tracks
    : library.getTracksByAlbum(props.album.name, props.album.artist)
  return [...raw].sort((a, b) => {
    const na = a.track_number != null ? parseInt(a.track_number, 10) : null
    const nb = b.track_number != null ? parseInt(b.track_number, 10) : null
    if (na == null && nb == null) return 0
    if (na == null) return 1
    if (nb == null) return -1
    return na - nb
  })
})

const coverTrack = computed(() => tracks.value.find((t) => t.has_cover))
watch(coverTrack, () => { coverFailed.value = false })

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
  <article :class="$style.card">
    <div :class="$style.coverWrapper">
      <button :class="$style.coverExpand" :aria-label="tr('{action} {name}', { action: expanded ? tr('Collapse') : tr('Expand'), name: album.name })" :aria-expanded="expanded" @click="expanded = !expanded">
        <img
          v-if="coverTrack && !coverFailed"
          :src="coverUrl(coverTrack.id)"
          alt=""
          :class="$style.coverImg"
          loading="lazy"
          @error="coverFailed = true"
        />
        <div v-else :class="$style.coverPlaceholder">&#9834;</div>
      </button>
      <div :class="$style.overlay">
        <button :class="[$style.iconBtn, $style.iconBtnPrimary]" @click.stop="playAlbum" :title="tr('Play album')">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="currentColor" stroke="none"><polygon points="6 3 20 12 6 21 6 3"></polygon></svg>
        </button>
        <button :class="$style.iconBtn" @click.stop="addAlbumToQueue" :title="tr('Add to queue')">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 12H3"></path><path d="M16 6H3"></path><path d="M16 18H3"></path><path d="M18 9v6"></path><path d="M21 12h-6"></path></svg>
        </button>
      </div>
    </div>
    <div :class="$style.details">
    <div :class="$style.info">
      <RouterLink
        :class="[$style.albumName, $style.albumNameLink]"
        :title="album.name"
        :to="{ name: 'album-detail', params: { name: album.name } }"
      >{{ album.name }}</RouterLink>
      <RouterLink
        v-if="album.artist"
        :class="[$style.albumMeta, $style.albumMetaLink]"
        :to="{ name: 'artist-detail', params: { name: album.artist } }"
      >{{ album.artist }}</RouterLink>
      <div :class="$style.albumMeta">
        {{ tr('{count} tracks', { count: album.track_count }) }} · {{ formatDurationLong(album.total_duration_secs) }}
      </div>
    </div>

    <button :class="$style.expandToggle" :aria-label="tr('{action} {name} tracks', { action: expanded ? tr('Collapse') : tr('Expand'), name: album.name })" :aria-expanded="expanded" @click="expanded = !expanded">
      <svg :class="[$style.chevron, expanded && $style.chevronOpen]" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"></path></svg>
    </button>
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
            :title="tr('Add to queue')"
            @click.stop="queue.addToQueue(track.id)"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 12H3"></path><path d="M16 6H3"></path><path d="M16 18H3"></path><path d="M18 9v6"></path><path d="M21 12h-6"></path></svg>
          </button>
        </div>
      </div>
    </Transition>
  </article>
</template>

<style module>
.card { min-width: 0; position: relative; }

.coverWrapper {
  position: relative;
  aspect-ratio: 1;
  overflow: hidden;
  cursor: pointer;
  border-radius: 12px;
  margin-bottom: 14px; box-shadow: 0 5px 14px var(--app-shadow);
}
.coverExpand { display: block; width: 100%; height: 100%; border: 0; padding: 0; cursor: pointer; background: transparent; color: inherit; }
.coverWrapper:focus-within { outline: 2px solid var(--app-focus-ring); outline-offset: 2px; }
.coverImg { width: 100%; height: 100%; object-fit: cover; display: block; transition: transform 0.2s ease; }
.coverWrapper:hover .coverImg { transform: scale(1.02); }
.coverPlaceholder {
  width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;
  background: var(--app-placeholder-bg);
  color: var(--app-text-muted);
  font-size: 48px;
}
.overlay {
  position: absolute; inset: 0;
  display: flex; align-items: flex-end; justify-content: flex-end; gap: 8px; padding: 12px;
  background: linear-gradient(transparent, rgba(0,0,0,0.5));
  opacity: 0; transition: opacity 0.2s;
  pointer-events: none;
}
.overlay button { pointer-events: auto; }
.coverWrapper:hover .overlay, .coverWrapper:focus-within .overlay { opacity: 1; }

.iconBtn {
  display: inline-flex; align-items: center; justify-content: center;
  width: 44px; height: 44px; border-radius: 50%;
  border: none; cursor: pointer; padding: 0;
  background: rgba(20,24,32,0.5); color: #fff;
  transition: background 0.15s, transform 0.1s;
}
.iconBtn:hover { background: rgba(255,255,255,0.3); transform: scale(1.08); }
.iconBtnPrimary { background: var(--app-primary); color: var(--app-on-primary); }
.iconBtnPrimary:hover { background: var(--app-primary-hover); }

.details { display: flex; align-items: flex-start; gap: 8px; }
.info { flex: 1; min-width: 0; padding: 0 2px; }
.albumName { font-weight: 600; font-size: 14px; line-height: 1.4; overflow: hidden; display: -webkit-box; -webkit-box-orient: vertical; -webkit-line-clamp: 2; }
.albumNameLink { cursor: pointer; color: inherit; text-decoration: none; }
.albumNameLink:hover { text-decoration: underline; opacity: 0.8; }
.albumMeta { font-size: 12px; color: var(--app-text-muted); margin-top: 5px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.albumMetaLink { cursor: pointer; display: block; text-decoration: none; }
.albumMetaLink:hover { text-decoration: underline; opacity: 0.8; }

.expandToggle {
  flex-shrink: 0; display: grid; place-items: center; width: 28px; height: 28px; padding: 0;
  border: 0; background: transparent; color: inherit;
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
.trackRowActive { color: var(--app-primary); font-weight: 600; }
.trackNum { width: 20px; text-align: right; opacity: 0.4; font-size: 11px; flex-shrink: 0; }
.trackTitle { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.trackDur { opacity: 0.5; font-size: 11px; font-variant-numeric: tabular-nums; flex-shrink: 0; }

.trackAddBtn {
  flex-shrink: 0; display: inline-flex; align-items: center; justify-content: center;
  width: 22px; height: 22px; border: none; border-radius: 50%;
  background: transparent; color: inherit; cursor: pointer; padding: 0;
  opacity: 0; transition: opacity 0.15s, background 0.15s;
}
.trackRow:hover .trackAddBtn, .trackRow:focus-within .trackAddBtn { opacity: 0.7; }
.trackAddBtn:hover { opacity: 1 !important; background: rgba(128,128,128,0.15); }
@media (hover: none) {
  .coverWrapper .overlay { opacity: 1; }
  .trackAddBtn { opacity: 0.7; }
}
</style>
