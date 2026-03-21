<script setup lang="ts">
import { computed } from 'vue'
import type { Track } from '@/types'
import { coverUrl } from '@/api/client'
import { formatDuration } from '@/utils/format'
import { useLibraryStore } from '@/stores/library'

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
    @click="emit('play')"
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
      <div :class="$style.itemTitle">{{ track?.title || 'Unknown' }}</div>
      <div :class="$style.itemArtist">{{ track?.artist || 'Unknown' }}</div>
    </div>
    <div :class="$style.itemDuration">
      {{ formatDuration(track?.duration_secs) }}
    </div>
    <button
      :class="$style.removeBtn"
      title="Remove from queue"
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
  padding: 8px 16px;
  cursor: pointer;
  transition: background 0.15s;
}

.item:hover {
  background: var(--n-merged-color, rgba(0, 0, 0, 0.04));
}

.itemActive {
  background: rgba(0, 102, 204, 0.08);
  border-left: 3px solid var(--n-primary-color, #0066cc);
}

.itemCover {
  width: 36px;
  height: 36px;
  border-radius: 4px;
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
  background: var(--n-merged-color, #f0f0f0);
  font-size: 14px;
  opacity: 0.3;
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
  opacity: 0.6;
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

.item:hover .removeBtn {
  opacity: 0.6;
}

.removeBtn:hover {
  opacity: 1 !important;
  background: rgba(128, 128, 128, 0.15);
}
</style>
