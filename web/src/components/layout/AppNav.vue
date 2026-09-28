<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { Music2, Disc3, Users, ListMusic, ChartNoAxesColumn } from 'lucide-vue-next'

const route = useRoute()
const items = [
  { label: 'Tracks', key: 'tracks', icon: Music2 },
  { label: 'Albums', key: 'albums', icon: Disc3 },
  { label: 'Artists', key: 'artists', icon: Users },
  { label: 'Playlists', key: 'playlists', icon: ListMusic },
  { label: 'Stats', key: 'stats', icon: ChartNoAxesColumn },
]
const detailSections: Record<string, string> = {
  'album-detail': 'albums',
  'artist-detail': 'artists',
  'playlist-detail': 'playlists',
}
const activeKey = computed(() => detailSections[String(route.name)] ?? route.name)
</script>

<template>
  <nav :class="$style.nav" aria-label="Library">
    <RouterLink
      v-for="item in items"
      :key="item.key"
      :to="{ name: item.key }"
      :class="[$style.link, activeKey === item.key && $style.active]"
      :aria-current="activeKey === item.key ? 'page' : undefined"
    >
      <component :is="item.icon" :size="16" :stroke-width="1.8" />
      <span>{{ item.label }}</span>
    </RouterLink>
  </nav>
</template>

<style module>
.nav { display: flex; gap: 4px; padding: 4px; border-radius: 12px; background: var(--app-inset); }
.link { display: flex; align-items: center; justify-content: center; gap: 8px; padding: 8px 16px; border-radius: 8px; color: var(--app-text-muted); font-size: 13px; font-weight: 500; text-decoration: none; transition: color 160ms, background 160ms, box-shadow 160ms; }
.link:hover { color: var(--app-text); }
.active { color: var(--app-text); background: var(--app-surface); box-shadow: 0 1px 4px var(--app-shadow); }
.link svg { flex-shrink: 0; }
.active svg { color: var(--app-primary); }
@media (max-width: 800px) {
  .nav { width: 100%; gap: 2px; }
  .link { flex: 1; min-width: 0; padding: 7px 3px; gap: 4px; font-size: 11px; flex-direction: column; }
}
</style>
