import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Track } from '@/types'
import { fetchTracks } from '@/api/tracks'

export const useLibraryStore = defineStore('library', () => {
  const allTracks = ref<Track[]>([])
  const searchQuery = ref('')
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const filteredTracks = computed(() => {
    if (!searchQuery.value) return allTracks.value
    const q = searchQuery.value.toLowerCase()
    return allTracks.value.filter(
      (t) =>
        t.title.toLowerCase().includes(q) ||
        t.artist.toLowerCase().includes(q) ||
        t.album.toLowerCase().includes(q),
    )
  })

  const totalTracks = computed(() => allTracks.value.length)

  function findTrack(id: string): Track | undefined {
    return allTracks.value.find((t) => t.id === id)
  }

  function updateTrackLocally(id: string, updates: Partial<Track>) {
    const track = allTracks.value.find((t) => t.id === id)
    if (track) {
      Object.assign(track, updates)
    }
  }

  async function loadTracks() {
    isLoading.value = true
    error.value = null
    try {
      allTracks.value = await fetchTracks()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load tracks'
    } finally {
      isLoading.value = false
    }
  }

  return {
    allTracks,
    searchQuery,
    isLoading,
    error,
    filteredTracks,
    totalTracks,
    findTrack,
    updateTrackLocally,
    loadTracks,
  }
})
