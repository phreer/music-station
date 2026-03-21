<script setup lang="ts">
import { NButton, NBadge } from 'naive-ui'
import { ListMusic } from 'lucide-vue-next'
import { useQueueStore } from '@/stores/queue'

const queue = useQueueStore()
</script>

<template>
  <div v-show="!queue.isEmpty" :class="$style.toggle">
    <NBadge :value="queue.queue.length" :max="99" :offset="[-4, 4]">
      <NButton circle type="primary" size="large" @click="queue.toggleVisible">
        <template #icon><ListMusic :size="20" /></template>
      </NButton>
    </NBadge>
  </div>
</template>

<style module>
.toggle {
  position: fixed;
  bottom: 100px;
  right: 24px;
  z-index: 80;
}
</style>
