import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Track } from '@/types'
import { usePlaylistStore } from './playlists'

export const useAddToPlaylistStore = defineStore('add-to-playlist', () => {
  const playlists = usePlaylistStore()
  const show = ref(false)
  const track = ref<Track | null>(null)
  const submittingId = ref<string | null>(null)
  const error = ref<string | null>(null)

  function open(selected: Track | null | undefined) {
    if (!selected || submittingId.value) return
    // Keep the selection independent of playback and later metadata changes.
    track.value = { ...selected, custom_fields: { ...selected.custom_fields } }
    error.value = null
    show.value = true
    void playlists.loadPlaylists()
  }

  function close() {
    if (submittingId.value) return
    show.value = false
    track.value = null
    error.value = null
  }

  async function add(playlistId: string): Promise<boolean> {
    const selected = track.value
    const playlist = playlists.findPlaylist(playlistId)
    if (!show.value || !selected || !playlist || playlists.isLoading || playlists.loadError
      || submittingId.value || playlist.tracks.includes(selected.id)) return false
    submittingId.value = playlistId
    error.value = null
    try {
      await playlists.addTrack(playlistId, selected.id)
      submittingId.value = null
      close()
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add to playlist'
      return false
    } finally {
      submittingId.value = null
    }
  }

  return { show, track, submittingId, error, open, close, add }
})
