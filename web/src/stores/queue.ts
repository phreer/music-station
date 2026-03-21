import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Track } from '@/types'
import { useLibraryStore } from './library'

export const useQueueStore = defineStore('queue', () => {
  const queue = ref<string[]>([])
  const currentIndex = ref(-1)
  const isVisible = ref(false)

  const library = useLibraryStore()

  const currentTrackId = computed(() => {
    if (currentIndex.value >= 0 && currentIndex.value < queue.value.length) {
      return queue.value[currentIndex.value]
    }
    return null
  })

  const queueTracks = computed<Track[]>(() => {
    return queue.value
      .map((id) => library.findTrack(id))
      .filter((t): t is Track => t !== undefined)
  })

  const isEmpty = computed(() => queue.value.length === 0)

  function addToQueue(trackId: string) {
    if (!queue.value.includes(trackId)) {
      queue.value.push(trackId)
    }
  }

  /**
   * Add a track to the queue (if not already present) and update currentIndex
   * to point at it, so the queue panel reflects the correct playing track.
   */
  function playInQueue(trackId: string) {
    if (!queue.value.includes(trackId)) {
      queue.value.push(trackId)
    }
    currentIndex.value = queue.value.indexOf(trackId)
  }

  function addMultiple(trackIds: string[]) {
    const existing = new Set(queue.value)
    const toAdd = trackIds.filter((id) => !existing.has(id))
    if (toAdd.length > 0) {
      queue.value.push(...toAdd)
    }
  }

  function removeFromQueue(index: number) {
    queue.value.splice(index, 1)
    if (index < currentIndex.value) {
      currentIndex.value--
    } else if (index === currentIndex.value) {
      // Current track removed; keep index, player will handle
      if (currentIndex.value >= queue.value.length) {
        currentIndex.value = queue.value.length - 1
      }
    }
  }

  function clear() {
    queue.value = []
    currentIndex.value = -1
  }

  function setQueue(trackIds: string[], startIndex = 0) {
    queue.value = [...trackIds]
    currentIndex.value = startIndex
  }

  function playIndex(index: number) {
    if (index >= 0 && index < queue.value.length) {
      currentIndex.value = index
    }
  }

  function next(): string | null {
    if (queue.value.length === 0) return null
    currentIndex.value = (currentIndex.value + 1) % queue.value.length
    return queue.value[currentIndex.value] ?? null
  }

  function previous(): string | null {
    if (queue.value.length === 0) return null
    currentIndex.value =
      currentIndex.value <= 0 ? queue.value.length - 1 : currentIndex.value - 1
    return queue.value[currentIndex.value] ?? null
  }

  function moveUp(index: number) {
    if (index <= 0) return
    const item = queue.value[index]!
    queue.value.splice(index, 1)
    queue.value.splice(index - 1, 0, item)
    if (currentIndex.value === index) currentIndex.value--
    else if (currentIndex.value === index - 1) currentIndex.value++
  }

  function moveDown(index: number) {
    if (index >= queue.value.length - 1) return
    const item = queue.value[index]!
    queue.value.splice(index, 1)
    queue.value.splice(index + 1, 0, item)
    if (currentIndex.value === index) currentIndex.value++
    else if (currentIndex.value === index + 1) currentIndex.value--
  }

  function toggleVisible() {
    isVisible.value = !isVisible.value
  }

  return {
    queue,
    currentIndex,
    isVisible,
    currentTrackId,
    queueTracks,
    isEmpty,
    addToQueue,
    playInQueue,
    addMultiple,
    removeFromQueue,
    clear,
    setQueue,
    playIndex,
    next,
    previous,
    moveUp,
    moveDown,
    toggleVisible,
  }
})
