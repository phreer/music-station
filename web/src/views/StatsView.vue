<script setup lang="ts">
import { computed, nextTick, onActivated, onDeactivated, ref } from 'vue'
import { NAlert, NButton, NSpin } from 'naive-ui'
import { Music, Disc3, Users, Clock, HardDrive, PlayCircle, RefreshCw, Headphones } from 'lucide-vue-next'
import { tr } from '@/i18n'
import LibraryPageHeader from '@/components/layout/LibraryPageHeader.vue'
import PlayRanking from '@/components/tracks/PlayRanking.vue'
import type { LibraryStats } from '@/types'
import { fetchStats } from '@/api/stats'
import { useLibraryStore } from '@/stores/library'
import { formatDurationLong, formatFileSize } from '@/utils/format'
import { listeningStats } from '@/utils/listeningStats'

const library = useLibraryStore()
const stats = ref<LibraryStats | null>(null)
const isLoading = ref(false)
const error = ref(false)
let controller: AbortController | null = null
const listening = computed(() => listeningStats(library.allTracks))
const busy = computed(() => isLoading.value || library.isLoading)
const listeningReady = computed(() => !library.error && (!library.isLoading || library.allTracks.length > 0))

async function refresh() {
  controller?.abort()
  const request = new AbortController()
  controller = request
  isLoading.value = true
  error.value = false
  try {
    await Promise.all([
      fetchStats(request.signal).then(value => { stats.value = value }),
      library.isLoading ? Promise.resolve() : library.loadTracks(),
    ])
  } catch {
    if (!request.signal.aborted) error.value = true
  } finally {
    if (controller === request) isLoading.value = false
  }
}

// The app starts the shared library request on mount; reuse it on a direct visit.
onActivated(async () => {
  await nextTick()
  void refresh()
})
onDeactivated(() => controller?.abort())

const statCards = computed(() => [
  { key: 'total_tracks' as const, label: tr('Tracks'), icon: Music, format: (v: number) => v.toLocaleString() },
  { key: 'total_albums' as const, label: tr('Albums'), icon: Disc3, format: (v: number) => v.toLocaleString() },
  { key: 'total_artists' as const, label: tr('Artists'), icon: Users, format: (v: number) => v.toLocaleString() },
  { key: 'total_duration_secs' as const, label: tr('Total Duration'), icon: Clock, format: formatDurationLong },
  { key: 'total_size_bytes' as const, label: tr('Library Size'), icon: HardDrive, format: formatFileSize },
])
</script>

<template>
  <div class="library-page">
    <LibraryPageHeader :title="tr('Your library, in numbers')" :description="tr('Your collection, your listening, your most-played tracks.')">
      <NButton secondary :loading="busy" @click="refresh">
        <template #icon><RefreshCw :size="16" /></template>
        {{ tr('Refresh stats') }}
      </NButton>
    </LibraryPageHeader>

    <NAlert v-if="error" type="error" :class="$style.error">
      {{ tr('Failed to load stats. Refresh to try again.') }}
    </NAlert>

    <NSpin :show="busy" :class="$style.content" :aria-busy="busy">
      <div :class="$style.overview">
        <section :class="$style.listening" aria-labelledby="listening-title">
          <div :class="$style.eyebrow"><Headphones :size="17" /><h2 id="listening-title">{{ tr('Listening overview') }}</h2></div>
          <div :class="$style.total">{{ listeningReady ? listening.totalPlays.toLocaleString() : '—' }}</div>
          <div :class="$style.totalLabel"><PlayCircle :size="16" />{{ tr('Total Plays') }} <span>· {{ tr('All time') }}</span></div>
          <div :class="$style.coverage">
            <div><span>{{ tr('Tracks played') }}</span><strong>{{ listeningReady ? listening.playedTracks.toLocaleString() : '—' }} <span>/ {{ library.allTracks.length.toLocaleString() }}</span></strong></div>
            <div :class="$style.coverageBar" aria-hidden="true"><span :style="{ width: `${listeningReady ? listening.coverage : 0}%` }" /></div>
            <p>{{ listeningReady ? tr('{percent}% of your library explored', { percent: listening.coverage }) : tr('Listening stats unavailable') }}</p>
          </div>
        </section>

        <section :class="$style.collection" aria-labelledby="collection-title">
          <h2 id="collection-title">{{ tr('Library overview') }}</h2>
          <div :class="$style.metrics">
            <div v-for="card in statCards" :key="card.key" :class="$style.metric">
              <component :is="card.icon" :size="18" :class="$style.metricIcon" />
              <div>
                <div :class="$style.statValue">{{ stats ? card.format(stats[card.key]) : '—' }}</div>
                <div :class="$style.statLabel">{{ card.label }}</div>
              </div>
            </div>
          </div>
        </section>
      </div>

      <PlayRanking :tracks="listening.rankedTracks" :disabled="busy || !!library.error">
        <template v-if="library.error" #default>
          <NAlert type="error" :class="$style.rankingError">
            {{ tr('Failed to load tracks') }}
            <NButton size="small" :loading="busy" @click="refresh">{{ tr('Retry') }}</NButton>
          </NAlert>
        </template>
        <template v-else-if="library.isLoading && !library.allTracks.length" #default>
          <p :class="$style.loading">{{ tr('Loading ranking…') }}</p>
        </template>
      </PlayRanking>
    </NSpin>
  </div>
</template>

<style module>
.content { display: block; min-height: 320px; }
.error { margin-bottom: 20px; }
.overview { display: grid; grid-template-columns: minmax(240px, 1fr) minmax(0, 2fr); gap: 20px; margin-bottom: 28px; }
.listening, .collection { border: 1px solid var(--app-border); border-radius: 16px; padding: 24px; }
.listening { background: linear-gradient(135deg, var(--app-active-bg), var(--app-surface)); }
.eyebrow { display: flex; align-items: center; gap: 8px; color: var(--app-primary); }
.eyebrow h2 { font-size: 13px; font-weight: 500; }
.total { font-size: 48px; line-height: 1.2; font-weight: 650; letter-spacing: -2px; margin: 22px 0 6px; overflow-wrap: anywhere; font-variant-numeric: tabular-nums; }
.totalLabel { display: flex; align-items: center; flex-wrap: wrap; gap: 6px; font-size: 13px; }
.totalLabel > span { color: var(--app-text-muted); }
.coverage { margin-top: 28px; }
.coverage > div:first-child { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 8px; font-size: 12px; }
.coverage strong { font-weight: 600; }
.coverage strong span, .coverage p { color: var(--app-text-muted); font-weight: 400; }
.coverage p { margin-top: 8px; font-size: 11px; }
.coverageBar { height: 5px; background: var(--app-border); border-radius: 8px; margin-top: 10px; overflow: hidden; }
.coverageBar > span { display: block; height: 100%; background: var(--app-primary); border-radius: inherit; }
.collection { background: var(--app-surface); }
.collection h2 { font-size: 14px; font-weight: 600; margin-bottom: 24px; }
.metrics { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 28px 20px; }
.metric { display: flex; align-items: flex-start; gap: 12px; min-width: 0; }
.metric > div { min-width: 0; }
.metricIcon { color: var(--app-text-muted); margin-top: 7px; flex-shrink: 0; }
.statValue { font-size: 24px; line-height: 1.35; font-weight: 600; letter-spacing: -.5px; overflow-wrap: anywhere; font-variant-numeric: tabular-nums; }
.statLabel { font-size: 12px; color: var(--app-text-muted); margin-top: 5px; }
.rankingError { margin: 0 24px 24px; }
.rankingError button { margin-left: 12px; }
.loading { padding: 48px 24px; text-align: center; color: var(--app-text-muted); }
@media (max-width: 1100px) {
  .metrics { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 22px 16px; }
  .statValue { font-size: 21px; }
}
@media (max-width: 700px) {
  .overview { grid-template-columns: minmax(0, 1fr); gap: 16px; margin-bottom: 20px; }
  .listening, .collection { padding: 20px; }
  .total { margin-top: 16px; font-size: 42px; }
  .coverage { margin-top: 20px; }
  .metrics { grid-template-columns: repeat(6, minmax(0, 1fr)); }
  .metric { grid-column: span 2; display: block; }
  .metric:nth-child(n + 4) { grid-column: span 3; }
  .metricIcon { margin: 0 0 8px; }
  .statValue { font-size: 18px; }
}
</style>
