<script setup lang="ts">
import { NEmpty, NScrollbar } from 'naive-ui'
import { useQueueStore } from '@/stores/queue'
import { usePlayerStore } from '@/stores/player'
import { formatDuration } from '@/utils/format'
import { computed } from 'vue'
import QueueItem from './QueueItem.vue'

const queue = useQueueStore()
const player = usePlayerStore()

const totalDuration = computed(() => {
  return queue.queueTracks.reduce((sum, t) => sum + (t.duration_secs ?? 0), 0)
})

const closeIconSvg = '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"></path><path d="m6 6 12 12"></path></svg>'
const trashIconSvg = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"></path><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path></svg>'

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
          <button
            :class="$style.headerBtn"
            :disabled="queue.isEmpty"
            title="Clear queue"
            @click="queue.clear"
          >
            <span v-html="trashIconSvg" />
            Clear
          </button>
          <button
            :class="$style.headerBtnCircle"
            title="Close"
            @click="queue.toggleVisible"
            v-html="closeIconSvg"
          />
        </div>
      </div>

      <div v-if="!queue.isEmpty" :class="$style.info">
        <span>{{ queue.queue.length }} tracks</span>
        <span>{{ formatDuration(totalDuration) }}</span>
      </div>

      <NScrollbar :class="$style.list">
        <NEmpty v-if="queue.isEmpty" description="Queue is empty" :class="$style.empty" />
        <QueueItem
          v-for="(trackId, index) in queue.queue"
          :key="trackId + '-' + index"
          :track-id="trackId"
          :is-active="index === queue.currentIndex"
          @play="playFromQueue(index)"
          @remove="remove(index)"
        />
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

.headerBtn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: inherit;
  cursor: pointer;
  padding: 4px 8px;
  font-size: 12px;
  opacity: 0.7;
  transition: opacity 0.15s, background 0.15s;
}

.headerBtn:hover:not(:disabled) {
  opacity: 1;
  background: rgba(128, 128, 128, 0.1);
}

.headerBtn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.headerBtnCircle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: inherit;
  cursor: pointer;
  padding: 0;
  opacity: 0.6;
  transition: opacity 0.15s, background 0.15s;
}

.headerBtnCircle:hover {
  opacity: 1;
  background: rgba(128, 128, 128, 0.15);
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
