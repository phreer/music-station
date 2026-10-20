<script setup lang="ts">
import { NButton, NSlider } from 'naive-ui'
import { Pause, Play, SkipBack, SkipForward } from 'lucide-vue-next'
import { usePlayerStore } from '@/stores/player'
import { formatDuration } from '@/utils/format'
import { tr } from '@/i18n'

const player = usePlayerStore()
</script>

<template>
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
      <NSlider :value="player.progress" :max="100" :step="0.1" :tooltip="false" :class="$style.slider" :aria-label="tr('Playback position')" @update:value="player.seek" />
      <span :class="$style.time">{{ formatDuration(player.duration) }}</span>
    </div>
  </div>
</template>

<style module>
.center { display: flex; flex-direction: column; align-items: center; gap: 4px; min-width: 0; width: 100%; }
.controls { display: flex; align-items: center; gap: 12px; }
.playButton { width: 38px; height: 38px; }
.progress { display: flex; align-items: center; gap: 10px; width: 100%; }
.slider { flex: 1; min-width: 0; }
.time { font-size: 10px; color: var(--app-text-muted); font-variant-numeric: tabular-nums; min-width: 28px; text-align: center; }
</style>
