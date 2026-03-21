<script setup lang="ts">
import { onMounted } from 'vue'
import { NSpin, NEmpty, NButton } from 'naive-ui'
import { RefreshCw } from 'lucide-vue-next'
import { useArtistsStore } from '@/stores/artists'
import ArtistGrid from '@/components/artists/ArtistGrid.vue'

const store = useArtistsStore()

onMounted(() => store.loadArtists())
</script>

<template>
  <div :class="$style.container">
    <div :class="$style.toolbar">
      <h2 :class="$style.heading">Artists</h2>
      <NButton quaternary circle @click="store.refresh" :loading="store.isLoading">
        <template #icon><RefreshCw :size="16" /></template>
      </NButton>
    </div>
    <NSpin :show="store.isLoading">
      <NEmpty v-if="!store.isLoading && store.allArtists.length === 0" description="No artists found" style="padding: 60px 0" />
      <ArtistGrid v-else :artists="store.allArtists" />
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
  gap: 8px;
  margin-bottom: 20px;
}
.heading {
  font-size: 22px;
  font-weight: 700;
  flex: 1;
}
</style>
