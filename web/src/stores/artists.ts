import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Artist } from '@/types'
import { fetchArtists } from '@/api/artists'

export const useArtistsStore = defineStore('artists', () => {
  const allArtists = ref<Artist[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  async function loadArtists() {
    if (allArtists.value.length > 0) return
    await _fetch()
  }

  async function refresh() {
    await _fetch()
  }

  async function _fetch() {
    isLoading.value = true
    error.value = null
    try {
      allArtists.value = await fetchArtists()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load artists'
    } finally {
      isLoading.value = false
    }
  }

  return { allArtists, isLoading, error, loadArtists, refresh }
})
