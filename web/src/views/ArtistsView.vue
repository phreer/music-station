<script setup lang="ts">
import { tr } from '@/i18n'
import LibraryPageHeader from '@/components/layout/LibraryPageHeader.vue'
import { ref, computed, onMounted } from 'vue'
import { NSpin, NEmpty, NButton, NInput } from 'naive-ui'
import { RefreshCw, Search, Heart } from 'lucide-vue-next'
import { useArtistsStore } from '@/stores/artists'
import ArtistGrid from '@/components/artists/ArtistGrid.vue'

const store = useArtistsStore()
const searchQuery = ref('')
const showFavoritesOnly = ref(false)

const favoriteCount = computed(() => store.allArtists.filter((a) => a.is_favorite).length)

const filteredArtists = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  let list = store.allArtists
  if (q) list = list.filter((a) => a.name.toLowerCase().includes(q))
  if (showFavoritesOnly.value) list = list.filter((a) => a.is_favorite)
  // Favorites first, preserve original order within each group
  return [...list].sort((a, b) => (b.is_favorite ? 1 : 0) - (a.is_favorite ? 1 : 0))
})

onMounted(() => store.loadArtists())
</script>

<template>
  <div class="library-page">
    <LibraryPageHeader :title="tr('Artists')" :description="tr('{count} artists in your library', { count: filteredArtists.length.toLocaleString() })">
      <NInput
        v-model:value="searchQuery"
        :placeholder="tr('Search artists')"
        clearable
        class="library-search"
      >
        <template #prefix>
          <Search :size="16" />
        </template>
      </NInput>
      <NButton
        :type="showFavoritesOnly ? 'primary' : 'default'"
        :secondary="!showFavoritesOnly"
        :class="$style.favBtn"
        @click="showFavoritesOnly = !showFavoritesOnly"
        :title="showFavoritesOnly ? tr('Show all artists') : tr('Show favorites only')"
      >
        <template #icon><Heart :size="14" /></template>
        {{ favoriteCount }}
      </NButton>
      <NButton quaternary circle :aria-label="tr('Refresh artists')" @click="store.refresh" :loading="store.isLoading">
        <template #icon><RefreshCw :size="16" /></template>
      </NButton>
    </LibraryPageHeader>
    <NSpin :show="store.isLoading">
      <NEmpty v-if="!store.isLoading && filteredArtists.length === 0" :description="tr('No artists found')" style="padding: 60px 0" />
      <ArtistGrid v-else :artists="filteredArtists" />
    </NSpin>
  </div>
</template>

<style module>
.favBtn { flex-shrink: 0; }
</style>
