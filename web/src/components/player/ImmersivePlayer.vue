<script setup lang="ts">
import { NButton, NModal, NSlider } from 'naive-ui'
import { AudioLines, Music2, Volume2, X } from 'lucide-vue-next'
import { usePlayerStore } from '@/stores/player'
import { useUiStore } from '@/stores/ui'
import { coverUrl } from '@/api/client'
import { tr } from '@/i18n'
import type { AudioAnalysis } from '@/audio/analysis'
import AudioVisualizer from './AudioVisualizer.vue'
import PlaybackControls from './PlaybackControls.vue'
import LyricsContent from '@/components/lyrics/LyricsContent.vue'

defineProps<{ analysis: AudioAnalysis; suspended: boolean }>()
const emit = defineEmits<{ toggleVisualizer: []; closed: [] }>()
const player = usePlayerStore()
const ui = useUiStore()
function close() { ui.immersiveVisible = false }
</script>

<template>
  <NModal :show="ui.immersiveVisible" :mask-closable="false" :trap-focus="true" :auto-focus="true"
    @update:show="ui.immersiveVisible = $event" @after-leave="emit('closed')">
    <!-- vueuc's focus trap locates the modal root by its DIV tag. -->
    <div :class="$style.view" role="dialog" aria-modal="true" :aria-label="tr('Immersive player')">
      <header :class="$style.header">
        <span>{{ tr('Now playing') }}</span>
        <NButton quaternary circle :aria-label="tr('Close immersive player')" @click="close"><template #icon><X :size="22" /></template></NButton>
      </header>
      <div :class="$style.main">
        <div :class="$style.song">
          <div :class="$style.cover">
            <img v-if="player.currentTrack?.has_cover" :src="coverUrl(player.currentTrack.id)" alt="" />
            <Music2 v-else :size="72" :stroke-width="1" />
          </div>
          <div :class="$style.metadata">
            <h1>{{ player.currentTrack?.title || tr('Unknown Title') }}</h1>
            <RouterLink v-if="player.currentTrack?.artist" :to="{ name: 'artist-detail', params: { name: player.currentTrack.artist } }" @click="close">{{ player.currentTrack.artist }}</RouterLink>
            <span v-else>{{ tr('Unknown Artist') }}</span>
            <RouterLink v-if="player.currentTrack?.album" :to="{ name: 'album-detail', params: { name: player.currentTrack.album } }" @click="close">{{ player.currentTrack.album }}</RouterLink>
          </div>
        </div>
        <div :class="$style.lyrics"><LyricsContent v-if="ui.immersiveVisible" immersive /></div>
      </div>
      <footer :class="$style.footer">
        <div v-if="ui.visualizerEnabled && ui.immersiveVisible" :class="$style.spectrum">
          <AudioVisualizer :analysis="analysis" :playing="player.isActuallyPlaying" :suspended="suspended" :dark="ui.isDarkMode" />
        </div>
        <div :class="$style.controls">
          <PlaybackControls />
          <div :class="$style.utilities">
            <div :class="$style.volume"><Volume2 :size="18" /><NSlider :value="player.volume" :max="1" :step="0.01" :tooltip="false" :aria-label="tr('Volume')" @update:value="player.setVolume" /></div>
            <NButton quaternary :type="ui.visualizerEnabled ? 'primary' : 'default'" :aria-pressed="ui.visualizerEnabled" @click="emit('toggleVisualizer')">
              <template #icon><AudioLines :size="18" /></template>{{ tr('Visualizer') }}
            </NButton>
          </div>
        </div>
      </footer>
    </div>
  </NModal>
</template>

<style module>
.view { width: 100vw; height: 100dvh; max-width: none; margin: 0; background: var(--app-bg); color: var(--app-text); display: flex; flex-direction: column; overflow: hidden; }
.header { display: flex; align-items: center; justify-content: space-between; padding: 20px 32px 8px; color: var(--app-text-muted); font-size: 13px; flex-shrink: 0; }
.main { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: clamp(20px, 5vw, 80px); flex: 1; min-height: 0; width: min(1100px, 100%); padding: 24px 36px; margin: 0 auto; }
.song { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 24px; min-height: 0; min-width: 0; }
.cover { width: min(100%, 360px, 35vh); aspect-ratio: 1; flex-shrink: 1; min-height: 0; border-radius: 20px; display: grid; place-items: center; overflow: hidden; background: var(--app-placeholder-bg); color: var(--app-text-muted); box-shadow: 0 20px 60px var(--app-shadow); }
.cover img { width: 100%; height: 100%; object-fit: cover; }
.metadata { text-align: center; display: flex; flex-direction: column; gap: 8px; max-width: 100%; overflow-wrap: anywhere; }
.metadata h1 { font-size: clamp(20px, 2.5vw, 30px); line-height: 1.3; }
.metadata a, .metadata span { color: var(--app-text-muted); font-size: 14px; text-decoration: none; }
.metadata a:hover { color: var(--app-primary); text-decoration: underline; }
.lyrics { min-height: 0; min-width: 0; display: flex; flex-direction: column; }
.footer { flex-shrink: 0; width: min(1000px, 100%); margin: 0 auto; padding: 0 32px calc(24px + env(safe-area-inset-bottom)); }
.spectrum { height: 112px; margin-bottom: 16px; }
.controls { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 32px; align-items: center; }
.utilities { display: flex; gap: 16px; align-items: center; }
.volume { display: flex; align-items: center; width: 120px; gap: 12px; }
@media (max-width: 700px) {
  .header { padding: 12px 16px 4px; }
  .main { display: flex; flex-direction: column; gap: 16px; padding: 12px 16px; }
  .song { flex-direction: row; justify-content: flex-start; flex-shrink: 0; gap: 16px; }
  .cover { width: clamp(64px, 17vh, 120px); flex-shrink: 0; border-radius: 12px; }
  .metadata { text-align: left; min-width: 0; gap: 4px; }
  .metadata h1 { font-size: 19px; }
  .metadata a { font-size: 12px; }
  .lyrics { flex: 1; }
  .footer { padding: 0 16px calc(12px + env(safe-area-inset-bottom)); background: var(--app-bg); }
  .spectrum { height: 56px; margin-bottom: 8px; }
  .controls { display: flex; flex-direction: column; gap: 6px; }
  .utilities { width: 100%; justify-content: space-between; }
  .volume { width: 140px; }
}
@media (max-height: 500px) and (min-width: 701px) {
  .main { padding-block: 8px; }
  .song { flex-direction: row; gap: 12px; }
  .cover { width: 100px; flex-shrink: 0; }
  .spectrum { height: 56px; margin-bottom: 4px; }
  .footer { padding-bottom: 12px; }
}
</style>
