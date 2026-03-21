<script setup lang="ts">
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
  <div :class="$style.container">
    <div :class="$style.toolbar">
      <NInput
        v-model:value="localSearchQuery"
        placeholder="Search tracks..."
        clearable
        :class="$style.searchInput"
      >
        <template #prefix>
          <Search :size="16" />
        </template>
      </NInput>
      <span :class="$style.trackCount">
        {{ library.filteredTracks.length }} / {{ library.totalTracks }} tracks
      </span>
      <NButton quaternary circle @click="refresh" :loading="library.isLoading">
        <template #icon>
          <RefreshCw :size="16" />
        </template>
      </NButton>
    </div>

    <NAlert v-if="library.error" type="error" :class="$style.error">
      {{ library.error }}
    </NAlert>

    <NSpin :show="library.isLoading && library.allTracks.length === 0">
      <TrackList :tracks="library.filteredTracks" />
    </NSpin>
  </div>
</template>

<style module>
.container {
  padding: 16px 24px;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.searchInput {
  max-width: 400px;
}

.trackCount {
  font-size: 13px;
  opacity: 0.6;
  white-space: nowrap;
}

.error {
  margin-bottom: 16px;
}
</style>
