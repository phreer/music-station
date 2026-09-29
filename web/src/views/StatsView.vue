<script setup lang="ts">
import { tr } from '@/i18n'
import LibraryPageHeader from '@/components/layout/LibraryPageHeader.vue'
import { computed, onMounted, ref } from 'vue'
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

const statCards = computed(() => [
  { key: 'total_tracks' as const, label: tr('Tracks'), icon: Music, format: (v: number) => v.toLocaleString() },
  { key: 'total_albums' as const, label: tr('Albums'), icon: Disc3, format: (v: number) => v.toLocaleString() },
  { key: 'total_artists' as const, label: tr('Artists'), icon: Users, format: (v: number) => v.toLocaleString() },
  { key: 'total_duration_secs' as const, label: tr('Total Duration'), icon: Clock, format: formatDurationLong },
  { key: 'total_size_bytes' as const, label: tr('Library Size'), icon: HardDrive, format: formatFileSize },
  { key: 'total_plays' as const, label: tr('Total Plays'), icon: PlayCircle, format: (v: number) => v.toLocaleString() },
])
</script>

<template>
  <div class="library-page">
    <LibraryPageHeader :title="tr('Your library, in numbers')" :description="tr('A closer look at your music collection.')" />
    <NSpin :show="isLoading">
      <NGrid v-if="stats" :x-gap="16" :y-gap="16" cols="2 900:3">
        <NGridItem v-for="card in statCards" :key="card.key">
          <NCard :class="$style.statCard">
            <div :class="$style.statIcon">
              <component :is="card.icon" :size="21" />
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
.statCard { padding: 4px 0; }
.statIcon { display: grid; place-items: center; width: 40px; height: 40px; border-radius: 12px; margin-bottom: 20px; color: var(--app-primary); background: var(--app-active-bg); }
.statValue { font-size: 30px; font-weight: 600; letter-spacing: -1px; margin-bottom: 4px; overflow-wrap: anywhere; }
.statLabel { font-size: 13px; color: var(--app-text-muted); }
@media (max-width: 600px) { .statValue { font-size: 23px; } }
</style>
