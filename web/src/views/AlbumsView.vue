<script setup lang="ts">
import { tr } from '@/i18n'
import LibraryPageHeader from '@/components/layout/LibraryPageHeader.vue'
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
  <div class="library-page">
    <LibraryPageHeader :title="tr('Albums')" :description="tr('{count} albums in your collection', { count: filteredAlbums.length.toLocaleString() })">
      <NInput
        v-model:value="searchQuery"
        :placeholder="tr('Search albums')"
        clearable
        class="library-search"
      >
        <template #prefix>
          <Search :size="16" />
        </template>
      </NInput>
      <NButton quaternary circle :aria-label="tr('Refresh albums')" @click="store.refresh" :loading="store.isLoading">
        <template #icon><RefreshCw :size="16" /></template>
      </NButton>
    </LibraryPageHeader>
    <NSpin :show="store.isLoading">
      <NEmpty v-if="!store.isLoading && filteredAlbums.length === 0" :description="tr('No albums found')" style="padding: 60px 0" />
      <AlbumGrid v-else :albums="filteredAlbums" />
    </NSpin>
  </div>
</template>
