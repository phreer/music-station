import { get } from './client'
import type { Artist } from '@/types'

export async function fetchArtists(): Promise<Artist[]> {
  return get<Artist[]>('/artists')
}

export async function fetchArtist(name: string): Promise<Artist> {
  return get<Artist>(`/artists/${encodeURIComponent(name)}`)
}
