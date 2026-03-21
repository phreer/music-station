import { get, putFormData, del } from './client'
import type { Lyrics, LyricsSearchResult } from '@/types'

export async function fetchLyrics(trackId: string): Promise<Lyrics> {
  return get<Lyrics>(`/lyrics/${trackId}`)
}

export async function uploadLyrics(
  trackId: string,
  content: string,
  format: string,
  language: string,
  source: string,
): Promise<void> {
  const formData = new FormData()
  formData.append('content', content)
  formData.append('format', format)
  if (language) formData.append('language', language)
  if (source) formData.append('source', source)
  await putFormData<void>(`/lyrics/${trackId}`, formData)
}

export async function deleteLyrics(trackId: string): Promise<void> {
  await del(`/lyrics/${trackId}`)
}

export async function searchLyrics(
  query: string,
  provider: string,
  artist?: string,
): Promise<LyricsSearchResult[]> {
  const params = new URLSearchParams({ q: query, provider })
  if (artist) params.set('artist', artist)
  return get<LyricsSearchResult[]>(`/lyrics/search?${params}`)
}

export async function fetchLyricsFromProvider(
  provider: string,
  songId: string,
): Promise<Lyrics> {
  return get<Lyrics>(`/lyrics/fetch/${provider}/${songId}`)
}
