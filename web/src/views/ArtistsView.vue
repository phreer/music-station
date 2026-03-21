<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { NSpin, NEmpty } from 'naive-ui'
import type { Artist } from '@/types'
import { fetchArtists } from '@/api/artists'
import ArtistGrid from '@/components/artists/ArtistGrid.vue'

const artists = ref<Artist[]>([])
const isLoading = ref(false)

onMounted(async () => {
  isLoading.value = true
  try {
    artists.value = await fetchArtists()
  } finally {
    isLoading.value = false
  }
})
</script>

<template>
  <div :class="$style.container">
    <h2 :class="$style.heading">Artists</h2>
    <NSpin :show="isLoading">
      <NEmpty v-if="!isLoading && artists.length === 0" description="No artists found" style="padding: 60px 0" />
      <ArtistGrid v-else :artists="artists" />
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
