<script setup lang="ts">
import { NMenu } from 'naive-ui'
import { computed, h } from 'vue'
import { Music, Disc3, Users, ListMusic, BarChart3 } from 'lucide-vue-next'
import { useUiStore, type ViewName } from '@/stores/ui'

const ui = useUiStore()

const menuOptions = [
  { label: 'Tracks', key: 'tracks' as ViewName, icon: Music },
  { label: 'Albums', key: 'albums' as ViewName, icon: Disc3 },
  { label: 'Artists', key: 'artists' as ViewName, icon: Users },
  { label: 'Playlists', key: 'playlists' as ViewName, icon: ListMusic },
  { label: 'Stats', key: 'stats' as ViewName, icon: BarChart3 },
]

const naiveMenuOptions = computed(() =>
  menuOptions.map((opt) => ({
    label: opt.label,
    key: opt.key,
    icon: () => h(opt.icon, { size: 16 }),
  })),
)

function handleSelect(key: string) {
  ui.switchView(key as ViewName)
}
</script>

<template>
  <nav :class="$style.nav">
    <NMenu
      mode="horizontal"
      :value="ui.currentView"
      :options="naiveMenuOptions"
      @update:value="handleSelect"
    />
  </nav>
</template>

<style module>
.nav {
  border-bottom: 1px solid var(--n-border-color, #e0e0e0);
}
</style>
