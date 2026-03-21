<script setup lang="ts">
import { NMenu } from 'naive-ui'
import { computed, h } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Music, Disc3, Users, ListMusic, BarChart3 } from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()

const menuOptions = [
  { label: 'Tracks', key: 'tracks', icon: Music },
  { label: 'Albums', key: 'albums', icon: Disc3 },
  { label: 'Artists', key: 'artists', icon: Users },
  { label: 'Playlists', key: 'playlists', icon: ListMusic },
  { label: 'Stats', key: 'stats', icon: BarChart3 },
]

const naiveMenuOptions = computed(() =>
  menuOptions.map((opt) => ({
    label: opt.label,
    key: opt.key,
    icon: () => h(opt.icon, { size: 16 }),
  })),
)

const activeKey = computed(() => {
  const name = route.name as string | undefined
  return name ?? 'tracks'
})

function handleSelect(key: string) {
  router.push({ name: key })
}
</script>

<template>
  <nav :class="$style.nav">
    <NMenu
      mode="horizontal"
      :value="activeKey"
      :options="naiveMenuOptions"
      @update:value="handleSelect"
    />
  </nav>
</template>

<style module>
.nav {
  border-bottom: 1px solid var(--app-border);
}
</style>
