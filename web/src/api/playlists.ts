import { get, post, put, del } from './client'
import type { Playlist } from '@/types'

export async function fetchPlaylists(): Promise<Playlist[]> {
  return get<Playlist[]>('/playlists')
}

export async function fetchPlaylist(id: string): Promise<Playlist> {
  return get<Playlist>(`/playlists/${id}`)
}

export async function createPlaylist(name: string, description?: string): Promise<Playlist> {
  return post<Playlist>('/playlists', { name, description })
}

export async function updatePlaylist(
  id: string,
  data: { name?: string; description?: string },
): Promise<Playlist> {
  return put<Playlist>(`/playlists/${id}`, data)
}

export async function deletePlaylist(id: string): Promise<void> {
  await del(`/playlists/${id}`)
}

export async function addTrackToPlaylist(playlistId: string, trackId: string): Promise<void> {
  await post<void>(`/playlists/${playlistId}/tracks/${trackId}`)
}

export async function removeTrackFromPlaylist(
  playlistId: string,
  trackId: string,
): Promise<void> {
  await del(`/playlists/${playlistId}/tracks/${trackId}`)
}
