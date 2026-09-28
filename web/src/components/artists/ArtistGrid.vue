<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount, onActivated, onDeactivated } from 'vue'
import type { Artist } from '@/types'
import ArtistCard from './ArtistCard.vue'

const BATCH_SIZE = 50

const props = defineProps<{ artists: Artist[] }>()

const visibleCount = ref(BATCH_SIZE)
const sentinel = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

const visibleArtists = computed(() => props.artists.slice(0, visibleCount.value))
const hasMore = computed(() => visibleCount.value < props.artists.length)

// Reset visible count when the artist list changes (e.g. refresh)
watch(() => props.artists, () => {
  visibleCount.value = BATCH_SIZE
})

function loadMore() {
  if (!hasMore.value) return
  visibleCount.value = Math.min(visibleCount.value + BATCH_SIZE, props.artists.length)
}

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
    <div :class="$style.grid">
      <ArtistCard v-for="artist in visibleArtists" :key="artist.name" :artist="artist" />
    </div>
    <div v-if="hasMore" ref="sentinel" style="height: 1px" />
  </div>
</template>

<style module>
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(100%, 300px), 1fr)); gap: 28px 24px; align-items: start; }
</style>
