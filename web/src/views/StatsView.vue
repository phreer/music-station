<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { NSpin, NGrid, NGridItem, NCard } from 'naive-ui'
import { Music, Disc3, Users, Clock, HardDrive, PlayCircle } from 'lucide-vue-next'
import type { LibraryStats } from '@/types'
import { fetchStats } from '@/api/stats'
import { formatDurationLong, formatFileSize } from '@/utils/format'

const stats = ref<LibraryStats | null>(null)
const isLoading = ref(false)

onMounted(async () => {
  isLoading.value = true
  try {
    stats.value = await fetchStats()
  } finally {
    isLoading.value = false
  }
})

const statCards = [
  { key: 'total_tracks' as const, label: 'Tracks', icon: Music, format: (v: number) => v.toLocaleString() },
  { key: 'total_albums' as const, label: 'Albums', icon: Disc3, format: (v: number) => v.toLocaleString() },
  { key: 'total_artists' as const, label: 'Artists', icon: Users, format: (v: number) => v.toLocaleString() },
  { key: 'total_duration_secs' as const, label: 'Total Duration', icon: Clock, format: formatDurationLong },
  { key: 'total_size_bytes' as const, label: 'Library Size', icon: HardDrive, format: formatFileSize },
  { key: 'total_plays' as const, label: 'Total Plays', icon: PlayCircle, format: (v: number) => v.toLocaleString() },
]
</script>

<template>
  <div :class="$style.container">
    <h2 :class="$style.heading">Library Stats</h2>
    <NSpin :show="isLoading">
      <NGrid v-if="stats" :x-gap="16" :y-gap="16" cols="2 600:3 900:6">
        <NGridItem v-for="card in statCards" :key="card.key">
          <NCard :class="$style.statCard">
            <div :class="$style.statIcon">
              <component :is="card.icon" :size="28" />
            </div>
            <div :class="$style.statValue">{{ card.format(stats[card.key]) }}</div>
            <div :class="$style.statLabel">{{ card.label }}</div>
          </NCard>
        </NGridItem>
      </NGrid>
    </NSpin>
  </div>
</template>

<style module>
.container {
  padding: 24px;
}
.heading {
  font-size: 22px;
  font-weight: 700;
  margin-bottom: 20px;
}
.statCard {
  text-align: center;
  padding: 8px 0;
}
.statIcon {
  display: flex;
  justify-content: center;
  margin-bottom: 8px;
  opacity: 0.5;
}
.statValue {
  font-size: 24px;
  font-weight: 700;
  margin-bottom: 4px;
}
.statLabel {
  font-size: 12px;
  opacity: 0.6;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
</style>
