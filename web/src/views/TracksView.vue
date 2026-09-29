<script setup lang="ts">
import { tr } from '@/i18n'
import LibraryPageHeader from '@/components/layout/LibraryPageHeader.vue'
import { ref, watch } from 'vue'
import { NInput, NSpin, NAlert, NButton } from 'naive-ui'
import { Search, RefreshCw } from 'lucide-vue-next'
import { useLibraryStore } from '@/stores/library'
import TrackList from '@/components/tracks/TrackList.vue'

const library = useLibraryStore()

// Local input value, debounced before updating the store's searchQuery.
// This avoids re-filtering the full track list on every keystroke.
const localSearchQuery = ref(library.searchQuery)
let debounceTimer: ReturnType<typeof setTimeout> | null = null

watch(localSearchQuery, (val) => {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    library.searchQuery = val
  }, 300)
})

function refresh() {
  library.loadTracks()
}
</script>

<template>
  <div class="library-page">
    <LibraryPageHeader :title="tr('Tracks')" :description="localSearchQuery ? tr('{count} of {total} tracks in your library', { count: library.filteredTracks.length.toLocaleString(), total: library.totalTracks.toLocaleString() }) : tr('{count} tracks in your library', { count: library.totalTracks.toLocaleString() })">
      <NInput
        v-model:value="localSearchQuery"
        :placeholder="tr('Search your music')"
        clearable
        class="library-search"
      >
        <template #prefix>
          <Search :size="16" />
        </template>
      </NInput>
      <NButton quaternary circle :aria-label="tr('Refresh tracks')" @click="refresh" :loading="library.isLoading">
        <template #icon>
          <RefreshCw :size="16" />
        </template>
      </NButton>
    </LibraryPageHeader>

    <NAlert v-if="library.error" type="error" :class="$style.error">
      {{ tr(library.error) }}
    </NAlert>

    <NSpin :show="library.isLoading && library.allTracks.length === 0">
      <TrackList :tracks="library.filteredTracks" />
    </NSpin>
  </div>
</template>

<style module>
.error { margin-bottom: 16px; }
</style>
