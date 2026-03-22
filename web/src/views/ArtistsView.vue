<script setup lang="ts">
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
  <div :class="$style.container">
    <div :class="$style.toolbar">
      <h2 :class="$style.heading">Artists</h2>
      <NInput
        v-model:value="searchQuery"
        placeholder="Search artists..."
        clearable
        :class="$style.searchInput"
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
        :title="showFavoritesOnly ? 'Show all artists' : 'Show favorites only'"
      >
        <template #icon><Heart :size="14" /></template>
        {{ favoriteCount }}
      </NButton>
      <span :class="$style.count">
        {{ filteredArtists.length }} / {{ store.allArtists.length }}
      </span>
      <NButton quaternary circle @click="store.refresh" :loading="store.isLoading">
        <template #icon><RefreshCw :size="16" /></template>
      </NButton>
    </div>
    <NSpin :show="store.isLoading">
      <NEmpty v-if="!store.isLoading && filteredArtists.length === 0" description="No artists found" style="padding: 60px 0" />
      <ArtistGrid v-else :artists="filteredArtists" />
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
.favBtn {
  flex-shrink: 0;
}
.count {
  font-size: 13px;
  opacity: 0.6;
  white-space: nowrap;
  flex-shrink: 0;
}
</style>
