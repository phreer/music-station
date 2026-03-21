import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Playlist } from '@/types'
import * as api from '@/api/playlists'

export const usePlaylistStore = defineStore('playlists', () => {
  const playlists = ref<Playlist[]>([])
  const isLoading = ref(false)
  let loadController: AbortController | null = null

  async function loadPlaylists() {
    loadController?.abort()
    loadController = new AbortController()
    isLoading.value = true
    try {
      playlists.value = await api.fetchPlaylists(loadController.signal)
    } catch (e) {
      if (e instanceof DOMException && e.name === 'AbortError') return
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

  return {
    playlists,
    isLoading,
    loadPlaylists,
    createPlaylist,
    deletePlaylist,
    addTrack,
    removeTrack,
    findPlaylist,
  }
})
