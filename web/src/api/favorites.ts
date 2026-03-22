import { get, put, del } from './client'

export interface FavoriteArtist {
  artist_name: string
  created_at: string
}

export async function fetchFavoriteArtists(signal?: AbortSignal): Promise<FavoriteArtist[]> {
  return get<FavoriteArtist[]>('/favorites/artists', signal)
}

export async function addFavoriteArtist(name: string, signal?: AbortSignal): Promise<void> {
  return put<void>(`/favorites/artists/${encodeURIComponent(name)}`, undefined, signal)
}

export async function removeFavoriteArtist(name: string, signal?: AbortSignal): Promise<void> {
  return del<void>(`/favorites/artists/${encodeURIComponent(name)}`, signal)
}
