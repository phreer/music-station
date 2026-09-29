<script setup lang="ts">
import { tr } from '@/i18n'
import { computed, ref, onMounted, watch } from 'vue'
import { NSlider, NButton, NDropdown, NPopover } from 'naive-ui'
import { Play, Pause, SkipBack, SkipForward, Volume2, VolumeX, MessageSquareText, Ellipsis, ListMusic, Music2 } from 'lucide-vue-next'
import { usePlayerStore } from '@/stores/player'
import { useQueueStore } from '@/stores/queue'
import { useLyricsStore } from '@/stores/lyrics'
import { coverUrl } from '@/api/client'
import { formatDuration } from '@/utils/format'
import LyricsModal from '@/components/modals/LyricsModal.vue'
import EditTrackModal from '@/components/modals/EditTrackModal.vue'
import TrackFavoriteButton from '@/components/tracks/TrackFavoriteButton.vue'

const player = usePlayerStore()
const lyrics = useLyricsStore()
const queue = useQueueStore()
const audioRef = ref<HTMLAudioElement | null>(null)

const showLyricsModal = ref(false)
const showEditModal = ref(false)

onMounted(() => {
  if (audioRef.value) {
    player.initAudio(audioRef.value)
  }
})

// Load lyrics when track changes
watch(
  () => player.currentTrackId,
  (trackId) => {
    if (trackId && player.currentTrack?.has_lyrics) {
      lyrics.loadForTrack(trackId)
    } else {
      lyrics.reset()
    }
  },
)

// Sync lyrics time
watch(
  () => player.currentTime,
  (time) => {
    if (lyrics.hasParsedLines) {
      lyrics.updateCurrentTime(time)
    }
  },
)

function handleSeek(value: number) {
  player.seek(value)
}
const trackActions = computed(() => [
  { label: tr('Edit track details'), key: 'edit' },
  { label: tr('Manage lyrics'), key: 'lyrics' },
  { type: 'divider', key: 'divider' },
  { label: tr('Stop playback'), key: 'stop' },
])

function handleAction(key: string) {
  if (key === 'edit') showEditModal.value = true
  if (key === 'lyrics') showLyricsModal.value = true
  if (key === 'stop') player.stop()
}
</script>


<template>
  <section v-show="player.currentTrack" :class="$style.player" :aria-label="tr('Music player')">
    <div :class="$style.info">
      <div :class="$style.cover">
        <img v-if="player.currentTrack?.has_cover" :src="coverUrl(player.currentTrack.id)" alt="" />
        <Music2 v-else :size="22" />
      </div>
      <div :class="$style.trackInfo">
        <div :class="$style.trackTitle" :title="player.currentTrack?.title ?? ''">{{ player.currentTrack?.title || tr('Unknown Title') }}</div>
        <div :class="$style.trackArtist">
          <RouterLink v-if="player.currentTrack?.artist" :to="{ name: 'artist-detail', params: { name: player.currentTrack.artist } }">{{ player.currentTrack.artist }}</RouterLink>
          <span v-else>{{ tr('Unknown Artist') }}</span>
          <template v-if="player.currentTrack?.album">
            <span :class="$style.separator"> / </span>
            <RouterLink :to="{ name: 'album-detail', params: { name: player.currentTrack.album } }">{{ player.currentTrack.album }}</RouterLink>
          </template>
        </div>
      </div>
      <TrackFavoriteButton v-if="player.currentTrack" :track-id="player.currentTrack.id" :size="32" :icon-size="17" />
      <NDropdown trigger="click" placement="top-start" :options="trackActions" @select="handleAction">
        <NButton quaternary circle :aria-label="tr('Track options')" :title="tr('Track options')"><template #icon><Ellipsis :size="19" /></template></NButton>
      </NDropdown>
    </div>

    <div :class="$style.center">
      <div :class="$style.controls">
        <NButton quaternary circle :aria-label="tr('Previous track')" @click="player.playPrevious"><template #icon><SkipBack :size="18" /></template></NButton>
        <NButton circle type="primary" :class="$style.playButton" :aria-label="player.isPlaying ? tr('Pause') : tr('Play')" @click="player.togglePlayPause">
          <template #icon><Pause v-if="player.isPlaying" :size="19" fill="currentColor" /><Play v-else :size="19" fill="currentColor" /></template>
        </NButton>
        <NButton quaternary circle :aria-label="tr('Next track')" @click="player.playNext"><template #icon><SkipForward :size="18" /></template></NButton>
      </div>
      <div :class="$style.progress">
        <span :class="$style.time">{{ formatDuration(player.currentTime) }}</span>
        <NSlider :value="player.progress" :max="100" :step="0.1" :tooltip="false" :class="$style.progressSlider" :aria-label="tr('Playback position')" @update:value="handleSeek" />
        <span :class="$style.time">{{ formatDuration(player.duration) }}</span>
      </div>
    </div>

    <div :class="$style.utilities">
      <div :class="$style.volumeDesktop">
        <Volume2 v-if="player.volume > 0" :size="17" /><VolumeX v-else :size="17" />
        <NSlider :value="player.volume" :max="1" :step="0.01" :tooltip="false" :aria-label="tr('Volume')" @update:value="player.setVolume" />
      </div>
      <div :class="$style.volumeMobile">
        <NPopover trigger="click" placement="top">
          <template #trigger><NButton quaternary circle :aria-label="tr('Adjust volume')"><template #icon><Volume2 v-if="player.volume > 0" :size="18" /><VolumeX v-else :size="18" /></template></NButton></template>
          <div :class="$style.volumePopover"><span>{{ tr('Volume') }}</span><NSlider :value="player.volume" :max="1" :step="0.01" :tooltip="false" :aria-label="tr('Volume')" @update:value="player.setVolume" /></div>
        </NPopover>
      </div>
      <NButton quaternary circle :class="lyrics.sidebarVisible && $style.utilityActive" :aria-pressed="lyrics.sidebarVisible" :aria-label="lyrics.sidebarVisible ? tr('Hide lyrics') : tr('Show lyrics')" @click="lyrics.toggleSidebar"><template #icon><MessageSquareText :size="18" /></template></NButton>
      <NButton id="queue-toggle-button" quaternary circle :class="queue.isVisible && $style.utilityActive" :aria-expanded="queue.isVisible" aria-controls="play-queue" :aria-label="queue.isVisible ? tr('Hide play queue') : tr('Show play queue')" @click="queue.toggleVisible"><template #icon><ListMusic :size="19" /></template></NButton>
    </div>
    <audio ref="audioRef" />
  </section>
  <LyricsModal v-model:show="showLyricsModal" :track="player.currentTrack" />
  <EditTrackModal v-model:show="showEditModal" :track="player.currentTrack" />
</template>

<style module>
.player { display: grid; grid-template-columns: minmax(0, 1fr) minmax(260px, 1.2fr) minmax(0, 1fr); align-items: center; gap: 24px; padding: 12px 20px; background: var(--app-surface-raised); border: 1px solid var(--app-border); border-radius: 16px; box-shadow: 0 8px 24px var(--app-shadow); }
.info { display: flex; align-items: center; gap: 10px; min-width: 0; }
.info > button { flex-shrink: 0; }
.cover { display: grid; place-items: center; width: 50px; height: 50px; border-radius: 9px; overflow: hidden; flex-shrink: 0; background: var(--app-placeholder-bg); color: var(--app-text-muted); }
.cover img { width: 100%; height: 100%; object-fit: cover; }
.trackInfo { flex: 1; min-width: 0; }
.trackTitle { font-weight: 600; font-size: 13px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.trackArtist { margin-top: 4px; font-size: 12px; color: var(--app-text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.trackArtist a { color: inherit; text-decoration: none; }
.trackArtist a:hover { color: var(--app-text); text-decoration: underline; }
.separator { margin: 0 2px; opacity: 0.5; }
.center { display: flex; flex-direction: column; align-items: center; gap: 4px; min-width: 0; }
.controls { display: flex; align-items: center; gap: 12px; }
.playButton { width: 38px; height: 38px; }
.progress { display: flex; align-items: center; gap: 10px; width: 100%; }
.progressSlider { flex: 1; min-width: 0; }
.time { font-size: 10px; color: var(--app-text-muted); font-variant-numeric: tabular-nums; min-width: 28px; text-align: center; }
.utilities { display: flex; align-items: center; justify-content: flex-end; gap: 8px; min-width: 0; }
.volumeDesktop { display: flex; align-items: center; gap: 10px; width: 110px; margin-right: 8px; color: var(--app-text-muted); }
.volumeDesktop svg { flex-shrink: 0; }
.volumeMobile { display: none; }
.volumePopover { width: 160px; display: grid; gap: 10px; padding: 4px; font-size: 12px; }
.utilityActive { color: var(--app-primary); background: var(--app-active-bg); }
@media (max-width: 1100px) {
  .player { grid-template-columns: minmax(0, 1fr) minmax(220px, 1fr) auto; gap: 16px; }
  .volumeDesktop { display: none; }
  .volumeMobile { display: block; }
}
@media (max-width: 700px) {
  .player { grid-template-columns: 1fr auto; padding: 12px 14px; gap: 8px 16px; border-radius: 14px; }
  .info { grid-column: 1 / -1; }
  .cover { width: 44px; height: 44px; }
  .center { display: contents; }
  .controls { grid-column: 1; grid-row: 2; gap: 8px; justify-content: flex-start; }
  .utilities { grid-column: 2; grid-row: 2; gap: 4px; }
  .progress { grid-column: 1 / -1; grid-row: 3; }
  .trackTitle { font-size: 14px; }
}
</style>
