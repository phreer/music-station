<script setup lang="ts">
import type { Track } from '@/types'
import { tr } from '@/i18n'
import { useAddToPlaylistStore } from '@/stores/addToPlaylist'

withDefaults(defineProps<{ track: Track | null | undefined; size?: number; iconSize?: number }>(), {
  size: 28,
  iconSize: 14,
})
const dialog = useAddToPlaylistStore()
</script>

<template>
  <button
    v-if="track"
    type="button"
    class="add-to-playlist-button"
    :style="{ width: `${size}px`, height: `${size}px` }"
    :title="tr('Add to playlist')"
    :aria-label="tr('Add to playlist')"
    @click.stop="dialog.open(track)"
  >
    <svg :width="iconSize" :height="iconSize" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d="M4 6h12M4 12h8M4 18h8M18 12v8M14 16h8" />
    </svg>
  </button>
</template>

<style scoped>
.add-to-playlist-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  padding: 0;
  border: 0;
  border-radius: 50%;
  background: transparent;
  color: inherit;
  cursor: pointer;
  opacity: 0.65;
}
.add-to-playlist-button:hover { opacity: 1; background: var(--app-hover); }
.add-to-playlist-button:focus-visible { opacity: 1; outline: 2px solid var(--app-focus-ring); outline-offset: 2px; }
@media (hover: none) { .add-to-playlist-button { opacity: 1; } }
</style>
