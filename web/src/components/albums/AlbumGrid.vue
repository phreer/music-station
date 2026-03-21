<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount, onActivated, onDeactivated } from 'vue'
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

// Walk up the DOM to find the nearest scrollable ancestor (the .main container).
// IntersectionObserver defaults to viewport as root, but the actual scrollable
// element is an overflow-y:auto div inside a height:100vh layout — so the sentinel
// would never intersect the viewport and loadMore() would never fire.
function findScrollParent(el: HTMLElement | null): HTMLElement | null {
  while (el) {
    if (/(auto|scroll)/.test(getComputedStyle(el).overflowY)) return el
    el = el.parentElement
  }
  return null
}

function setupObserver() {
  observer?.disconnect()
  const root = findScrollParent(sentinel.value)
  observer = new IntersectionObserver(
    (entries) => {
      if (entries[0]?.isIntersecting) loadMore()
    },
    { root, rootMargin: '200px' },
  )
  if (sentinel.value) observer.observe(sentinel.value)
}

// Re-observe when the sentinel element is (re-)created by v-if after a visibleCount reset
watch(sentinel, (el) => {
  if (el && observer) observer.observe(el)
})

onMounted(setupObserver)
onActivated(setupObserver)
onDeactivated(() => observer?.disconnect())
onBeforeUnmount(() => observer?.disconnect())
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
