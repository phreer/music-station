import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Album } from '@/types'
import { fetchAlbums } from '@/api/albums'

export const useAlbumsStore = defineStore('albums', () => {
  const allAlbums = ref<Album[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  let loadController: AbortController | null = null

  async function loadAlbums() {
    if (allAlbums.value.length > 0) return
    await _fetch()
  }

  async function refresh() {
    await _fetch()
  }

  async function _fetch() {
    loadController?.abort()
    loadController = new AbortController()
    isLoading.value = true
    error.value = null
    try {
      allAlbums.value = await fetchAlbums(loadController.signal)
    } catch (e) {
      if (e instanceof DOMException && e.name === 'AbortError') return
      error.value = e instanceof Error ? e.message : 'Failed to load albums'
    } finally {
      isLoading.value = false
    }
  }

  return { allAlbums, isLoading, error, loadAlbums, refresh }
})
