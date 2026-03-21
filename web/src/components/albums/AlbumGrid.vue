<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { NGrid, NGridItem } from 'naive-ui'
import type { Album } from '@/types'
import AlbumCard from './AlbumCard.vue'

const BATCH_SIZE = 50

const props = defineProps<{ albums: Album[] }>()

const visibleCount = ref(BATCH_SIZE)
const sentinel = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

const visibleAlbums = computed(() => props.albums.slice(0, visibleCount.value))
const hasMore = computed(() => visibleCount.value < props.albums.length)

// Reset visible count when the album list changes (e.g. refresh)
watch(() => props.albums, () => {
  visibleCount.value = BATCH_SIZE
})

function loadMore() {
  if (!hasMore.value) return
  visibleCount.value = Math.min(visibleCount.value + BATCH_SIZE, props.albums.length)
}

onMounted(() => {
  observer = new IntersectionObserver(
    (entries) => {
      if (entries[0]?.isIntersecting) loadMore()
    },
    { rootMargin: '200px' },
  )
  if (sentinel.value) observer.observe(sentinel.value)
})

onBeforeUnmount(() => {
  observer?.disconnect()
})
</script>

<template>
  <div>
    <NGrid :x-gap="16" :y-gap="16" cols="1 500:2 800:3 1100:4">
      <NGridItem v-for="album in visibleAlbums" :key="album.name + album.artist">
        <AlbumCard :album="album" />
      </NGridItem>
    </NGrid>
    <div v-if="hasMore" ref="sentinel" style="height: 1px" />
  </div>
</template>
