import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Track } from '@/types'
import { fetchTracks } from '@/api/tracks'

export const useLibraryStore = defineStore('library', () => {
  const allTracks = ref<Track[]>([])
  const trackMap = ref(new Map<string, Track>())
  const searchQuery = ref('')
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const filteredTracks = computed(() => {
    if (!searchQuery.value) return allTracks.value
    const q = searchQuery.value.toLowerCase()
    return allTracks.value.filter(
      (t) =>
        t.title?.toLowerCase().includes(q) ||
        t.artist?.toLowerCase().includes(q) ||
        t.album?.toLowerCase().includes(q),
    )
  })

  const totalTracks = computed(() => allTracks.value.length)

  // Pre-computed artist-to-tracks index — replaces per-card allTracks.filter()
  const tracksByArtist = computed(() => {
    const map = new Map<string, Track[]>()
    for (const t of allTracks.value) {
      for (const name of [t.artist, t.album_artist]) {
        if (name) {
          let arr = map.get(name)
          if (!arr) {
            arr = []
            map.set(name, arr)
          }
          arr.push(t)
        }
      }
    }
    return map
  })

  // Pre-computed album-to-tracks index (keyed by "album\0artist")
  const tracksByAlbumKey = computed(() => {
    const map = new Map<string, Track[]>()
    for (const t of allTracks.value) {
      if (t.album) {
        const key = t.album + '\0' + (t.artist ?? '')
        let arr = map.get(key)
        if (!arr) {
          arr = []
          map.set(key, arr)
        }
        arr.push(t)
      }
    }
    return map
  })

  function findTrack(id: string): Track | undefined {
    return trackMap.value.get(id)
  }

  function getTracksByArtist(name: string): Track[] {
    return tracksByArtist.value.get(name) ?? []
  }

  function getTracksByAlbum(albumName: string, artistName: string | null): Track[] {
    return tracksByAlbumKey.value.get(albumName + '\0' + (artistName ?? '')) ?? []
  }

  function updateTrackLocally(id: string, updates: Partial<Track>) {
    const track = trackMap.value.get(id)
    if (track) {
      Object.assign(track, updates)
    }
  }

  async function loadTracks() {
    isLoading.value = true
    error.value = null
    try {
      const tracks = await fetchTracks()
      allTracks.value = tracks
      const map = new Map<string, Track>()
      for (const t of tracks) map.set(t.id, t)
      trackMap.value = map
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
    getTracksByArtist,
    getTracksByAlbum,
    updateTrackLocally,
    loadTracks,
  }
})
