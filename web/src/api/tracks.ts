import { get, put, post } from './client'
import type { Track } from '@/types'

export async function fetchTracks(): Promise<Track[]> {
  return get<Track[]>('/tracks')
}

export async function fetchTrack(id: string): Promise<Track> {
  return get<Track>(`/tracks/${id}`)
}

export async function updateTrack(id: string, data: Partial<Track>): Promise<Track> {
  return put<Track>(`/tracks/${id}`, data)
}

export async function incrementPlayCount(id: string): Promise<void> {
  await post<void>(`/tracks/${id}/play`)
}
