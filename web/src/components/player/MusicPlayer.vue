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
import { useRouter } from 'vue-router'
import { usePlayerStore } from '@/stores/player'
import { useLyricsStore } from '@/stores/lyrics'
import { coverUrl } from '@/api/client'
import { formatDuration } from '@/utils/format'
import LyricsModal from '@/components/modals/LyricsModal.vue'
import EditTrackModal from '@/components/modals/EditTrackModal.vue'
import TrackFavoriteButton from '@/components/tracks/TrackFavoriteButton.vue'

const player = usePlayerStore()
const lyrics = useLyricsStore()
const router = useRouter()
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
          <span
            v-if="player.currentTrack?.artist"
            :class="$style.trackLink"
            @click="router.push({ name: 'artist-detail', params: { name: player.currentTrack.artist } })"
          >{{ player.currentTrack.artist }}</span>
          <span v-if="player.currentTrack?.artist && player.currentTrack?.album" :class="$style.trackSep"> · </span>
          <span
            v-if="player.currentTrack?.album"
            :class="$style.trackLink"
            @click="router.push({ name: 'album-detail', params: { name: player.currentTrack.album } })"
          >{{ player.currentTrack.album }}</span>
          <span v-if="!player.currentTrack?.artist && !player.currentTrack?.album">Unknown Artist</span>
        </div>
        </div>
        <TrackFavoriteButton
          v-if="player.currentTrack"
          :track-id="player.currentTrack.id"
          :size="30"
          :icon-size="15"
        />
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
        <NButton quaternary circle size="small" aria-label="Previous track" @click="player.playPrevious">
          <template #icon><SkipBack :size="16" /></template>
        </NButton>
        <NButton circle type="primary" :class="$style.playButton" :aria-label="player.isPlaying ? 'Pause' : 'Play'" @click="player.togglePlayPause">
          <template #icon>
            <Pause v-if="player.isPlaying" :size="20" />
            <Play v-else :size="20" />
          </template>
        </NButton>
        <NButton quaternary circle size="small" aria-label="Next track" @click="player.playNext">
          <template #icon><SkipForward :size="16" /></template>
        </NButton>
        <NButton quaternary circle size="small" aria-label="Stop" @click="player.stop">
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
  min-height: 80px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  padding: 10px 20px;
  gap: 18px;
  background: var(--app-surface-raised);
  color: var(--app-text);
  z-index: 100;
  border: 1px solid var(--app-border);
  border-radius: 16px;
  box-shadow: 0 10px 30px var(--app-shadow);
}

.info {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1 1 270px;
  min-width: 0;
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
  border-radius: 8px;
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
  background: var(--app-placeholder-bg);
  font-size: 20px;
  color: var(--app-text-muted);
}

.trackInfo {
  min-width: 0;
  flex: 1;
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
  color: var(--app-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.trackLink {
  cursor: pointer;
}

.trackLink:hover {
  text-decoration: underline;
  opacity: 1;
}

.trackSep {
  opacity: 0.5;
}

.center {
  flex: 2 1 360px;
  min-width: 0;
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

.playButton { width: 40px; height: 40px; }

.progress {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  max-width: 600px;
}

.progressSlider {
  flex: 1;
  min-width: 0;
}

.time {
  font-size: 11px;
  color: var(--app-text-muted);
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
  color: var(--app-text-muted);
}

.volumeSlider {
  flex: 1;
}

@media (max-width: 900px) {
  .player { flex-wrap: wrap; gap: 4px 16px; padding: 10px 14px; }
  .info { flex: 1 1 calc(100% - 170px); }
  .center { order: 2; flex: 1 1 100%; }
  .volume { flex: 0 0 130px; }
}

@media (max-width: 520px) {
  .player { gap: 6px 10px; }
  .info { flex-basis: calc(100% - 125px); gap: 8px; }
  .cover { width: 42px; height: 42px; }
  .trackActions { gap: 0; }
  .volume { width: 110px; flex-basis: 110px; }
  .progress { gap: 4px; }
}
</style>
