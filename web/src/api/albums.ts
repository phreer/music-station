import { get } from './client'
import type { Album } from '@/types'

export async function fetchAlbums(): Promise<Album[]> {
  return get<Album[]>('/albums')
}

export async function fetchAlbum(name: string): Promise<Album> {
  return get<Album>(`/albums/${encodeURIComponent(name)}`)
}
