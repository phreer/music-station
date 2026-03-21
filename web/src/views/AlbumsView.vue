<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { NSpin, NEmpty } from 'naive-ui'
import type { Album } from '@/types'
import { fetchAlbums } from '@/api/albums'
import AlbumGrid from '@/components/albums/AlbumGrid.vue'

const albums = ref<Album[]>([])
const isLoading = ref(false)
const error = ref<string | null>(null)

onMounted(async () => {
  isLoading.value = true
  try {
    albums.value = await fetchAlbums()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load albums'
  } finally {
    isLoading.value = false
  }
})
</script>

<template>
  <div :class="$style.container">
    <h2 :class="$style.heading">Albums</h2>
    <NSpin :show="isLoading">
      <NEmpty v-if="!isLoading && albums.length === 0" description="No albums found" style="padding: 60px 0" />
      <AlbumGrid v-else :albums="albums" />
    </NSpin>
  </div>
</template>

<style module>
.container {
  padding: 24px;
}
.heading {
  font-size: 22px;
  font-weight: 700;
  margin-bottom: 20px;
}
</style>
