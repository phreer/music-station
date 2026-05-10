import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import type { Playlist } from '@/types'
import * as api from '@/api/playlists'

const FAVORITE_PLAYLIST_NAME = 'Favorite'

export const usePlaylistStore = defineStore('playlists', () => {
  const playlists = ref<Playlist[]>([])
  const isLoading = ref(false)
  const hasLoaded = ref(false)
  const favoritePendingTrackIds = ref<Set<string>>(new Set())
  let loadController: AbortController | null = null

  const favoritePlaylist = computed(() =>
    playlists.value.find((playlist) => playlist.name === FAVORITE_PLAYLIST_NAME),
  )

  async function loadPlaylists() {
    loadController?.abort()
    loadController = new AbortController()
    isLoading.value = true
    try {
      playlists.value = await api.fetchPlaylists(loadController.signal)
      hasLoaded.value = true
    } catch (e) {
      if (e instanceof DOMException && e.name === 'AbortError') return
      hasLoaded.value = true
    } finally {
      isLoading.value = false
    }
  }

  async function createPlaylist(name: string, description?: string) {
    const playlist = await api.createPlaylist(name, description)
    playlists.value.push(playlist)
    return playlist
  }

  async function deletePlaylist(id: string) {
    await api.deletePlaylist(id)
    playlists.value = playlists.value.filter((p) => p.id !== id)
  }

  async function addTrack(playlistId: string, trackId: string) {
    await api.addTrackToPlaylist(playlistId, trackId)
    const playlist = playlists.value.find((p) => p.id === playlistId)
    if (playlist && !playlist.tracks.includes(trackId)) {
      playlist.tracks.push(trackId)
    }
  }

  async function removeTrack(playlistId: string, trackId: string) {
    await api.removeTrackFromPlaylist(playlistId, trackId)
    const playlist = playlists.value.find((p) => p.id === playlistId)
    if (playlist) {
      playlist.tracks = playlist.tracks.filter((id) => id !== trackId)
    }
  }

  function findPlaylist(id: string): Playlist | undefined {
    return playlists.value.find((p) => p.id === id)
  }

  async function ensureFavoritePlaylist(): Promise<Playlist> {
    if (!hasLoaded.value) {
      await loadPlaylists()
    }

    const existing = favoritePlaylist.value
    if (existing) return existing

    return createPlaylist(FAVORITE_PLAYLIST_NAME, 'Tracks you hearted')
  }

  function isTrackFavorited(trackId: string): boolean {
    return favoritePlaylist.value?.tracks.includes(trackId) ?? false
  }

  function isTrackFavoritePending(trackId: string): boolean {
    return favoritePendingTrackIds.value.has(trackId)
  }

  async function toggleTrackFavorite(trackId: string): Promise<void> {
    if (isTrackFavoritePending(trackId)) return

    setFavoritePending(trackId, true)
    try {
      const playlist = await ensureFavoritePlaylist()
      if (isTrackFavorited(trackId)) {
        await removeTrack(playlist.id, trackId)
      } else {
        await addTrack(playlist.id, trackId)
      }
    } finally {
      setFavoritePending(trackId, false)
    }
  }

  function setFavoritePending(trackId: string, pending: boolean) {
    const next = new Set(favoritePendingTrackIds.value)
    if (pending) {
      next.add(trackId)
    } else {
      next.delete(trackId)
    }
    favoritePendingTrackIds.value = next
  }

  return {
    playlists,
    isLoading,
    favoritePlaylist,
    loadPlaylists,
    createPlaylist,
    deletePlaylist,
    addTrack,
    removeTrack,
    findPlaylist,
    ensureFavoritePlaylist,
    isTrackFavorited,
    isTrackFavoritePending,
    toggleTrackFavorite,
  }
})
