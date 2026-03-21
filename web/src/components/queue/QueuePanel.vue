<script setup lang="ts">
import { NButton, NEmpty, NScrollbar } from 'naive-ui'
import { X, Trash2 } from 'lucide-vue-next'
import { useQueueStore } from '@/stores/queue'
import { usePlayerStore } from '@/stores/player'
import { useLibraryStore } from '@/stores/library'
import { coverUrl } from '@/api/client'
import { formatDuration } from '@/utils/format'
import { computed } from 'vue'

const queue = useQueueStore()
const player = usePlayerStore()
const library = useLibraryStore()

const totalDuration = computed(() => {
  return queue.queueTracks.reduce((sum, t) => sum + (t.duration_secs ?? 0), 0)
})

function playFromQueue(index: number) {
  queue.playIndex(index)
  const trackId = queue.queue[index]
  if (trackId) {
    player.playTrack(trackId)
  }
}

function remove(index: number) {
  queue.removeFromQueue(index)
}
</script>

<template>
  <Transition name="slide">
    <div v-if="queue.isVisible" :class="$style.panel">
      <div :class="$style.header">
        <h3 :class="$style.title">Play Queue</h3>
        <div :class="$style.headerActions">
          <NButton quaternary size="tiny" @click="queue.clear" :disabled="queue.isEmpty">
            <template #icon><Trash2 :size="14" /></template>
            Clear
          </NButton>
          <NButton quaternary circle size="small" @click="queue.toggleVisible">
            <template #icon><X :size="16" /></template>
          </NButton>
        </div>
      </div>

      <div v-if="!queue.isEmpty" :class="$style.info">
        <span>{{ queue.queue.length }} tracks</span>
        <span>{{ formatDuration(totalDuration) }}</span>
      </div>

      <NScrollbar :class="$style.list">
        <NEmpty v-if="queue.isEmpty" description="Queue is empty" :class="$style.empty" />
        <div
          v-for="(trackId, index) in queue.queue"
          :key="trackId + '-' + index"
          :class="[$style.item, index === queue.currentIndex && $style.itemActive]"
          @click="playFromQueue(index)"
        >
          <div :class="$style.itemCover">
            <img
              v-if="library.findTrack(trackId)?.has_cover"
              :src="coverUrl(trackId)"
              :class="$style.coverImg"
            />
            <div v-else :class="$style.coverPlaceholder">&#9834;</div>
          </div>
          <div :class="$style.itemInfo">
            <div :class="$style.itemTitle">
              {{ library.findTrack(trackId)?.title || 'Unknown' }}
            </div>
            <div :class="$style.itemArtist">
              {{ library.findTrack(trackId)?.artist || 'Unknown' }}
            </div>
          </div>
          <div :class="$style.itemDuration">
            {{ formatDuration(library.findTrack(trackId)?.duration_secs) }}
          </div>
          <NButton
            quaternary
            circle
            size="tiny"
            :class="$style.removeBtn"
            @click.stop="remove(index)"
          >
            <template #icon><X :size="12" /></template>
          </NButton>
        </div>
      </NScrollbar>
    </div>
  </Transition>
</template>

<style module>
.panel {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 80px;
  width: 350px;
  display: flex;
  flex-direction: column;
  background: var(--n-color, #fff);
  border-left: 1px solid var(--n-border-color, #e0e0e0);
  z-index: 90;
  box-shadow: -4px 0 20px rgba(0, 0, 0, 0.1);
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--n-border-color, #e0e0e0);
}

.title {
  font-size: 16px;
  font-weight: 600;
}

.headerActions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.info {
  display: flex;
  justify-content: space-between;
  padding: 8px 16px;
  font-size: 12px;
  opacity: 0.6;
  border-bottom: 1px solid var(--n-border-color, #e0e0e0);
}

.list {
  flex: 1;
}

.empty {
  padding: 40px 0;
}

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
  opacity: 0;
  transition: opacity 0.15s;
}

.item:hover .removeBtn {
  opacity: 1;
}
</style>

<style>
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.25s ease;
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
}
</style>
