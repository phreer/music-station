<script setup lang="ts">
import { computed } from 'vue'
import { Heart } from 'lucide-vue-next'
import { usePlaylistStore } from '@/stores/playlists'

const props = withDefaults(defineProps<{
  trackId: string
  size?: number
  iconSize?: number
}>(), {
  size: 28,
  iconSize: 14,
})

const playlists = usePlaylistStore()

const isFavorite = computed(() => playlists.isTrackFavorited(props.trackId))
const isPending = computed(() => playlists.isTrackFavoritePending(props.trackId))
const buttonTitle = computed(() =>
  isFavorite.value ? 'Remove from Favorite playlist' : 'Add to Favorite playlist',
)

async function toggleFavorite(event: MouseEvent) {
  event.stopPropagation()
  await playlists.toggleTrackFavorite(props.trackId)
}
</script>

<template>
  <button
    :class="['track-favorite-button', isFavorite && 'is-active']"
    :style="{ width: `${size}px`, height: `${size}px` }"
    :title="buttonTitle"
    :aria-pressed="isFavorite"
    :disabled="isPending"
    @click="toggleFavorite"
  >
    <Heart :size="iconSize" :fill="isFavorite ? 'currentColor' : 'none'" />
  </button>
</template>

<style>
.track-favorite-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 999px;
  background: transparent;
  color: inherit;
  cursor: pointer;
  padding: 0;
  opacity: 0.62;
  transition: opacity 0.15s, background 0.15s, color 0.15s;
}

.track-favorite-button:hover:not(:disabled) {
  opacity: 1;
  background: rgba(128, 128, 128, 0.15);
}

.track-favorite-button.is-active {
  color: #e05c7a;
  opacity: 1;
}

.track-favorite-button:disabled {
  cursor: wait;
  opacity: 0.45;
}
</style>
