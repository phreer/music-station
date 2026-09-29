<script setup lang="ts">
import { tr } from '@/i18n'
import { computed } from 'vue'
import type { Track } from '@/types'
import { coverUrl } from '@/api/client'
import { formatDuration } from '@/utils/format'
import { useLibraryStore } from '@/stores/library'
import TrackFavoriteButton from '@/components/tracks/TrackFavoriteButton.vue'

const props = defineProps<{
  trackId: string
  isActive: boolean
}>()

const emit = defineEmits<{
  play: []
  remove: []
}>()

const library = useLibraryStore()

// Resolve track data once per component instance
const track = computed<Track | undefined>(() => library.findTrack(props.trackId))

const removeIconSvg = '<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"></path><path d="m6 6 12 12"></path></svg>'
</script>

<template>
  <div
    :class="[$style.item, isActive && $style.itemActive]"
    tabindex="0"
    :aria-label="tr('Play {title}', { title: track?.title || tr('Unknown') })"
    @click="emit('play')"
    @keydown.enter.self="emit('play')"
    @keydown.space.self.prevent="emit('play')"
  >
    <div :class="$style.itemCover">
      <img
        v-if="track?.has_cover"
        :src="coverUrl(trackId)"
        :class="$style.coverImg"
      />
      <div v-else :class="$style.coverPlaceholder">&#9834;</div>
    </div>
    <div :class="$style.itemInfo">
      <div :class="$style.itemTitle">{{ track?.title || tr('Unknown') }}</div>
      <div :class="$style.itemArtist">{{ track?.artist || tr('Unknown') }}</div>
    </div>
    <div :class="$style.itemDuration">
      {{ formatDuration(track?.duration_secs) }}
    </div>
    <TrackFavoriteButton v-if="track" :track-id="track.id" :size="22" :icon-size="12" />
    <button
      :class="$style.removeBtn"
      :title="tr('Remove from queue')"
      @click.stop="emit('remove')"
      v-html="removeIconSvg"
    />
  </div>
</template>

<style module>
.item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  cursor: pointer;
  transition: background 0.15s;
}

.item:hover {
  background: var(--app-hover);
}

.itemActive {
  background: var(--app-active-bg);
  box-shadow: inset 3px 0 var(--app-primary);
}

.itemCover {
  width: 36px;
  height: 36px;
  border-radius: 7px;
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
  font-size: 14px;
  color: var(--app-text-muted);
}

.itemInfo {
  flex: 1;
  min-width: 0;
}

.itemTitle {
  font-size: 13px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.itemArtist {
  font-size: 11px;
  color: var(--app-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.itemDuration {
  font-size: 11px;
  opacity: 0.5;
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}

.removeBtn {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: inherit;
  cursor: pointer;
  padding: 0;
  opacity: 0;
  transition: opacity 0.15s, background 0.15s;
}

.item :global(.track-favorite-button) {
  flex-shrink: 0;
  opacity: 0;
}

.item:hover .removeBtn,
.item:focus-within .removeBtn,
.item:hover :global(.track-favorite-button),
.item:focus-within :global(.track-favorite-button) {
  opacity: 0.6;
}

@media (hover: none) {
  .item .removeBtn, .item :global(.track-favorite-button) { opacity: 0.7; }
}

.removeBtn:hover {
  opacity: 1 !important;
  background: rgba(128, 128, 128, 0.15);
}

.removeBtn:focus-visible, .item :global(.track-favorite-button.is-active) { opacity: 1; }
</style>
