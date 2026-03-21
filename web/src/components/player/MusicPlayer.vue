<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { NSlider, NButton } from 'naive-ui'
import {
  Play,
  Pause,
  SkipBack,
  SkipForward,
  Square,
  Volume2,
  VolumeX,
  Music2,
  Pencil,
} from 'lucide-vue-next'
import { usePlayerStore } from '@/stores/player'
import { useLyricsStore } from '@/stores/lyrics'
import { coverUrl } from '@/api/client'
import { formatDuration } from '@/utils/format'
import LyricsModal from '@/components/modals/LyricsModal.vue'
import EditTrackModal from '@/components/modals/EditTrackModal.vue'

const player = usePlayerStore()
const lyrics = useLyricsStore()
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
</script>

<template>
  <div v-show="player.currentTrack" :class="$style.player">
    <!-- Track Info -->
    <div :class="$style.info">
      <div :class="$style.cover">
        <img
          v-if="player.currentTrack?.has_cover"
          :src="coverUrl(player.currentTrack!.id)"
          :class="$style.coverImg"
        />
        <div v-else :class="$style.coverPlaceholder">&#9834;</div>
      </div>
      <div :class="$style.trackInfo">
        <div :class="$style.trackTitle">
          {{ player.currentTrack?.title || 'Unknown Title' }}
        </div>
        <div :class="$style.trackArtist">
          {{ player.currentTrack?.artist || 'Unknown Artist' }}
        </div>
      </div>
      <!-- Per-track actions -->
      <div :class="$style.trackActions">
        <NButton
          quaternary
          circle
          size="small"
          :title="'Edit metadata'"
          @click="showEditModal = true"
        >
          <template #icon><Pencil :size="13" /></template>
        </NButton>
        <NButton
          quaternary
          circle
          size="small"
          :title="'Manage lyrics'"
          :type="lyrics.hasLyrics ? 'primary' : 'default'"
          @click="showLyricsModal = true"
        >
          <template #icon><Music2 :size="13" /></template>
        </NButton>
        <NButton
          quaternary
          circle
          size="small"
          :title="lyrics.sidebarVisible ? 'Hide lyrics sidebar' : 'Show lyrics sidebar'"
          @click="lyrics.toggleSidebar"
        >
          <template #icon><Music2 :size="13" style="opacity: 0.5" /></template>
        </NButton>
      </div>
    </div>

    <!-- Controls -->
    <div :class="$style.center">
      <div :class="$style.controls">
        <NButton quaternary circle size="small" @click="player.playPrevious">
          <template #icon><SkipBack :size="16" /></template>
        </NButton>
        <NButton quaternary circle @click="player.togglePlayPause">
          <template #icon>
            <Pause v-if="player.isPlaying" :size="20" />
            <Play v-else :size="20" />
          </template>
        </NButton>
        <NButton quaternary circle size="small" @click="player.playNext">
          <template #icon><SkipForward :size="16" /></template>
        </NButton>
        <NButton quaternary circle size="small" @click="player.stop">
          <template #icon><Square :size="14" /></template>
        </NButton>
      </div>
      <div :class="$style.progress">
        <span :class="$style.time">{{ formatDuration(player.currentTime) }}</span>
        <NSlider
          :value="player.progress"
          :max="100"
          :step="0.1"
          :tooltip="false"
          :class="$style.progressSlider"
          @update:value="handleSeek"
        />
        <span :class="$style.time">{{ formatDuration(player.duration) }}</span>
      </div>
    </div>

    <!-- Volume -->
    <div :class="$style.volume">
      <Volume2 v-if="player.volume > 0" :size="16" />
      <VolumeX v-else :size="16" />
      <NSlider
        :value="player.volume"
        :max="1"
        :step="0.01"
        :tooltip="false"
        :class="$style.volumeSlider"
        @update:value="player.setVolume"
      />
    </div>

    <audio ref="audioRef" />
  </div>

  <!-- Lyrics Modal -->
  <LyricsModal v-model:show="showLyricsModal" :track="player.currentTrack" />
  <!-- Edit Modal -->
  <EditTrackModal v-model:show="showEditModal" :track="player.currentTrack" />
</template>

<style module>
.player {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: 80px;
  display: flex;
  align-items: center;
  padding: 0 24px;
  gap: 24px;
  background: linear-gradient(135deg, #1a2332, #0f1419);
  color: #e8f4f8;
  z-index: 100;
  border-top: 2px solid #0066cc;
}

.info {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 280px;
  flex-shrink: 0;
}

.trackActions {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
}

.cover {
  width: 48px;
  height: 48px;
  border-radius: 6px;
  overflow: hidden;
  flex-shrink: 0;
}

.coverImg {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.coverPlaceholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.1);
  font-size: 20px;
  opacity: 0.5;
}

.trackInfo {
  min-width: 0;
}

.trackTitle {
  font-weight: 600;
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.trackArtist {
  font-size: 12px;
  opacity: 0.7;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.center {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.progress {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  max-width: 600px;
}

.progressSlider {
  flex: 1;
}

.time {
  font-size: 11px;
  opacity: 0.7;
  font-variant-numeric: tabular-nums;
  width: 40px;
  text-align: center;
}

.volume {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 150px;
  flex-shrink: 0;
  opacity: 0.7;
}

.volumeSlider {
  flex: 1;
}
</style>
