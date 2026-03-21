import { get } from './client'
import type { Album } from '@/types'

export async function fetchAlbums(signal?: AbortSignal): Promise<Album[]> {
  return get<Album[]>('/albums', signal)
}

export async function fetchAlbum(name: string, signal?: AbortSignal): Promise<Album> {
  return get<Album>(`/albums/${encodeURIComponent(name)}`, signal)
}
