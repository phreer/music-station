<script setup lang="ts">
import { onMounted } from 'vue'
import { NSpin, NEmpty, NButton } from 'naive-ui'
import { RefreshCw } from 'lucide-vue-next'
import { useAlbumsStore } from '@/stores/albums'
import AlbumGrid from '@/components/albums/AlbumGrid.vue'

const store = useAlbumsStore()

onMounted(() => store.loadAlbums())
</script>

<template>
  <div :class="$style.container">
    <div :class="$style.toolbar">
      <h2 :class="$style.heading">Albums</h2>
      <NButton quaternary circle @click="store.refresh" :loading="store.isLoading">
        <template #icon><RefreshCw :size="16" /></template>
      </NButton>
    </div>
    <NSpin :show="store.isLoading">
      <NEmpty v-if="!store.isLoading && store.allAlbums.length === 0" description="No albums found" style="padding: 60px 0" />
      <AlbumGrid v-else :albums="store.allAlbums" />
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
