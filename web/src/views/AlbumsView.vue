<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { NSpin, NEmpty, NButton, NInput } from 'naive-ui'
import { RefreshCw, Search } from 'lucide-vue-next'
import { useAlbumsStore } from '@/stores/albums'
import AlbumGrid from '@/components/albums/AlbumGrid.vue'

const store = useAlbumsStore()
const searchQuery = ref('')

const filteredAlbums = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return store.allAlbums
  return store.allAlbums.filter(
    (a) =>
      a.name.toLowerCase().includes(q) ||
      (a.artist && a.artist.toLowerCase().includes(q)),
  )
})

onMounted(() => store.loadAlbums())
</script>

<template>
  <div :class="$style.container">
    <div :class="$style.toolbar">
      <h2 :class="$style.heading">Albums</h2>
      <NInput
        v-model:value="searchQuery"
        placeholder="Search albums..."
        clearable
        :class="$style.searchInput"
      >
        <template #prefix>
          <Search :size="16" />
        </template>
      </NInput>
      <span :class="$style.count">
        {{ filteredAlbums.length }} / {{ store.allAlbums.length }}
      </span>
      <NButton quaternary circle @click="store.refresh" :loading="store.isLoading">
        <template #icon><RefreshCw :size="16" /></template>
      </NButton>
    </div>
    <NSpin :show="store.isLoading">
      <NEmpty v-if="!store.isLoading && filteredAlbums.length === 0" description="No albums found" style="padding: 60px 0" />
      <AlbumGrid v-else :albums="filteredAlbums" />
    </NSpin>
  </div>
</template>

<style module>
.container {
  padding: 24px;
}
.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}
.heading {
  font-size: 22px;
  font-weight: 700;
  flex-shrink: 0;
}
.searchInput {
  max-width: 320px;
  flex: 1;
}
.count {
  font-size: 13px;
  opacity: 0.6;
  white-space: nowrap;
  flex-shrink: 0;
}
</style>
