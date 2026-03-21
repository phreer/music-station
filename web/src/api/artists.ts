import { get } from './client'
import type { Artist } from '@/types'

export async function fetchArtists(signal?: AbortSignal): Promise<Artist[]> {
  return get<Artist[]>('/artists', signal)
}

export async function fetchArtist(name: string, signal?: AbortSignal): Promise<Artist> {
  return get<Artist>(`/artists/${encodeURIComponent(name)}`, signal)
}
