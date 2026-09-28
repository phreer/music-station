<script setup lang="ts">
import { NButton, NBadge } from 'naive-ui'
import { ListMusic } from 'lucide-vue-next'
import { usePlayerStore } from '@/stores/player'
import { useQueueStore } from '@/stores/queue'

const queue = useQueueStore()
const player = usePlayerStore()
</script>

<template>
  <div v-if="!queue.isEmpty && !player.currentTrack" :class="$style.toggle">
    <NBadge :value="queue.queue.length" :max="99" :offset="[-4, 4]">
      <NButton id="queue-toggle-button" circle type="primary" size="large" :aria-label="queue.isVisible ? 'Hide play queue' : 'Show play queue'" @click="queue.toggleVisible">
        <template #icon><ListMusic :size="20" /></template>
      </NButton>
    </NBadge>
  </div>
</template>

<style module>
.toggle {
  position: fixed;
  bottom: calc(var(--app-player-height) + 20px);
  right: 24px;
  z-index: 80;
}
</style>
