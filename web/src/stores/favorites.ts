import { defineStore } from 'pinia'
import { addFavoriteArtist, removeFavoriteArtist } from '@/api/favorites'
import { useArtistsStore } from './artists'

export const useFavoritesStore = defineStore('favorites', () => {
  const artistsStore = useArtistsStore()

  /** Toggle favorite status for an artist. Updates local state optimistically. */
  async function toggleArtist(artistName: string): Promise<void> {
    const artist = artistsStore.allArtists.find((a) => a.name === artistName)
    if (!artist) return

    const wasFavorite = artist.is_favorite
    // Optimistic update
    artist.is_favorite = !wasFavorite

    try {
      if (wasFavorite) {
        await removeFavoriteArtist(artistName)
      } else {
        await addFavoriteArtist(artistName)
      }
    } catch (e) {
      // Roll back on failure
      artist.is_favorite = wasFavorite
      throw e
    }
  }

  return { toggleArtist }
})
