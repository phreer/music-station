<script setup lang="ts">
import { useAddToPlaylistStore } from '@/stores/addToPlaylist'
import { tr } from '@/i18n'
import { computed, ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { NSlider, NButton, NDropdown, NPopover, useMessage } from 'naive-ui'
import { Volume2, VolumeX, MessageSquareText, Ellipsis, ListMusic, Music2, AudioLines } from 'lucide-vue-next'
import { usePlayerStore } from '@/stores/player'
import { useQueueStore } from '@/stores/queue'
import { useLyricsStore } from '@/stores/lyrics'
import { coverUrl } from '@/api/client'
import { useUiStore } from '@/stores/ui'
import { AudioAnalysis } from '@/audio/analysis'
import AudioVisualizer from './AudioVisualizer.vue'
import PlaybackControls from './PlaybackControls.vue'
import ImmersivePlayer from './ImmersivePlayer.vue'
import LyricsModal from '@/components/modals/LyricsModal.vue'
import EditTrackModal from '@/components/modals/EditTrackModal.vue'
import TrackFavoriteButton from '@/components/tracks/TrackFavoriteButton.vue'

const dialog = useAddToPlaylistStore()
const player = usePlayerStore()
const lyrics = useLyricsStore()
const queue = useQueueStore()
const ui = useUiStore()
const message = useMessage()
const analysisReady = ref(false)
const pageVisible = ref(document.visibilityState !== 'hidden')
const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)')
const analysis = new AudioAnalysis(
  () => { analysisReady.value = analysis.ready },
  () => { message.warning(tr('Visualizer unavailable. Try playing again to resume audio.')) },
)
const suspended = computed(() => !pageVisible.value || player.isWaiting || !analysisReady.value)
const ambientSuspended = computed(() => suspended.value || ui.immersiveVisible || !ui.visualizerEnabled || !player.currentTrack)
const ambientVisible = computed(() => !ambientSuspended.value && player.isActuallyPlaying)
const ambientMounted = ref(false)

// Keep props live during the leave transition so pausing stops sampling immediately.
watch([ambientVisible, ambientSuspended], ([visible, stopped]) => {
  if (visible) ambientMounted.value = true
  else if (stopped) ambientMounted.value = false
}, { immediate: true })

function finishAmbientLeave() {
  if (!ambientVisible.value) ambientMounted.value = false
}

const coverRef = ref<HTMLButtonElement | null>(null)
let immersiveOpener: HTMLElement | null = null

function openImmersive(event: MouseEvent) {
  immersiveOpener = event.currentTarget as HTMLElement
  ui.immersiveVisible = true
}

function restoreFocus() {
  if (!ui.immersiveVisible) (immersiveOpener?.isConnected ? immersiveOpener : coverRef.value)?.focus()
}

function prepareAudio() {
  if (audioRef.value) void analysis.prepare(audioRef.value, ui.visualizerEnabled)
}

function toggleVisualizer() {
  ui.setVisualizerEnabled(!ui.visualizerEnabled)
}

function visibilityChanged() { pageVisible.value = document.visibilityState !== 'hidden' }
function motionChanged() {
  if (reducedMotion.matches) ui.setVisualizerEnabled(false, false)
}

watch(() => ui.visualizerEnabled, (enabled) => {
  if (enabled) {
    analysis.budget.reset()
    prepareAudio()
  }
}, { flush: 'sync' })
const audioRef = ref<HTMLAudioElement | null>(null)

const showLyricsModal = ref(false)
const showEditModal = ref(false)

onMounted(() => {
  if (audioRef.value) {
    player.initAudio(audioRef.value, prepareAudio)
  }
  document.addEventListener('visibilitychange', visibilityChanged)
  reducedMotion.addEventListener('change', motionChanged)
})

onBeforeUnmount(() => {
  document.removeEventListener('visibilitychange', visibilityChanged)
  reducedMotion.removeEventListener('change', motionChanged)
  player.destroyAudio()
  analysis.destroy()
  ui.immersiveVisible = false
})

// Load lyrics when track changes
watch(
  () => player.currentTrackId,
  (trackId) => {
    if (!trackId) ui.immersiveVisible = false
    if (trackId && player.currentTrack?.has_lyrics) {
      lyrics.loadForTrack(trackId)
    } else {
      lyrics.reset()
    }
  },
)

// Sync lyrics time
watch(
  () => [player.currentTime, lyrics.parsedLines] as const,
  ([time]) => {
    if (lyrics.hasParsedLines) {
      lyrics.updateCurrentTime(time)
    }
  },
)

const trackActions = computed(() => [
  ...(player.currentTrack ? [{ label: tr('Add to playlist'), key: 'playlist' }] : []),
  { label: tr('Edit track details'), key: 'edit' },
  { label: tr('Manage lyrics'), key: 'lyrics' },
  { type: 'divider', key: 'divider' },
  { label: tr('Stop playback'), key: 'stop' },
])

function handleAction(key: string) {
  if (key === 'playlist') dialog.open(player.currentTrack)
  if (key === 'edit') showEditModal.value = true
  if (key === 'lyrics') showLyricsModal.value = true
  if (key === 'stop') player.stop()
}
</script>


<template>
  <section v-show="player.currentTrack" :class="$style.player" :aria-label="tr('Music player')">
    <Transition
      :enter-active-class="$style.ambientEnter"
      :leave-active-class="$style.ambientLeave"
      :enter-from-class="$style.ambientHidden"
      :leave-to-class="$style.ambientHidden"
      @after-leave="finishAmbientLeave"
    >
      <div v-show="ambientVisible" :class="[$style.ambient, ui.isDarkMode && $style.ambientDark]" aria-hidden="true">
        <AudioVisualizer v-if="ambientMounted" :class="$style.ambientCanvas" variant="ambient"
          :analysis="analysis" :playing="ambientVisible" :suspended="ambientSuspended" :dark="ui.isDarkMode" />
      </div>
    </Transition>
    <div :class="$style.info">
      <button ref="coverRef" type="button" :class="$style.cover" :aria-label="tr('Open immersive player')" @click="openImmersive">
        <img v-if="player.currentTrack?.has_cover" :src="coverUrl(player.currentTrack.id)" alt="" />
        <Music2 v-else :size="22" />
      </button>
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

    <PlaybackControls :class="$style.playback" />

    <div :class="$style.utilities">
      <NButton
        quaternary
        circle
        :class="ui.visualizerEnabled && $style.utilityActive"
        :aria-pressed="ui.visualizerEnabled"
        :aria-label="ui.visualizerEnabled ? tr('Hide visualizer') : tr('Show visualizer')"
        :title="ui.visualizerEnabled ? tr('Hide visualizer') : tr('Show visualizer')"
        @click="toggleVisualizer"
      >
        <template #icon><AudioLines :size="18" /></template>
      </NButton>
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
  <ImmersivePlayer :analysis="analysis" :suspended="suspended" @toggle-visualizer="toggleVisualizer" @closed="restoreFocus" />
  <LyricsModal v-model:show="showLyricsModal" :track="player.currentTrack" />
  <EditTrackModal v-model:show="showEditModal" :track="player.currentTrack" />
</template>

<style module>
.player { position: relative; isolation: isolate; display: grid; grid-template-columns: minmax(0, 1fr) minmax(260px, 1.2fr) minmax(0, 1fr); align-items: center; gap: 24px; padding: 12px 20px; background: var(--app-surface-raised); border: 1px solid var(--app-border); border-radius: 16px; box-shadow: 0 8px 24px var(--app-shadow); }
.ambient { position: absolute; inset: 0; z-index: 0; overflow: hidden; border-radius: inherit; pointer-events: none; opacity: 0.28; }
.ambientDark { opacity: 0.38; }
.ambient .ambientCanvas { position: absolute; inset: auto 0 0; height: min(75%, 64px); mask-image: linear-gradient(to right, transparent, #000 8%, #000 92%, transparent); }
.ambientEnter { transition: opacity 300ms ease-out; }
.ambientLeave { transition: opacity 250ms ease-in; }
.ambientHidden { opacity: 0; }
.info, .playback, .utilities { position: relative; z-index: 1; }
.info { display: flex; align-items: center; gap: 10px; min-width: 0; }
.info > button { flex-shrink: 0; }
.cover { display: grid; place-items: center; width: 50px; height: 50px; border-radius: 9px; overflow: hidden; flex-shrink: 0; background: var(--app-placeholder-bg); color: var(--app-text-muted); border: 0; cursor: pointer; }
.cover img { width: 100%; height: 100%; object-fit: cover; }
.trackInfo { flex: 1; min-width: 0; }
.trackTitle { font-weight: 600; font-size: 13px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.trackArtist { margin-top: 4px; font-size: 12px; color: var(--app-text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.trackArtist a { color: inherit; text-decoration: none; }
.trackArtist a:hover { color: var(--app-text); text-decoration: underline; }
.separator { margin: 0 2px; opacity: 0.5; }
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
  .playback { grid-column: 1 / -1; grid-row: 2; }
  .utilities { grid-column: 1 / -1; grid-row: 3; gap: 4px; justify-content: center; }
  .ambient .ambientCanvas { height: min(75%, 56px); }
  .trackTitle { font-size: 14px; }
}
@media (prefers-reduced-motion: reduce) {
  .ambientEnter, .ambientLeave { transition: none; }
}
</style>
