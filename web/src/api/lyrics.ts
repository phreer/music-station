import { get, put, del } from './client'
import type { Lyrics, LyricsSearchResult } from '@/types'

export async function fetchLyrics(trackId: string, signal?: AbortSignal): Promise<Lyrics> {
  return get<Lyrics>(`/lyrics/${trackId}`, signal)
}

export async function uploadLyrics(
  trackId: string,
  content: string,
  format: string,
  language?: string,
  source?: string,
): Promise<void> {
  const body: Record<string, string> = { content, format }
  if (language) body.language = language
  if (source) body.source = source
  await put<void>(`/lyrics/${trackId}`, body)
}

export async function deleteLyrics(trackId: string): Promise<void> {
  await del(`/lyrics/${trackId}`)
}

export async function searchLyrics(
  query: string,
  provider: string,
  artist?: string,
  signal?: AbortSignal,
): Promise<LyricsSearchResult[]> {
  const params = new URLSearchParams({ q: query, provider })
  if (artist) params.set('artist', artist)
  return get<LyricsSearchResult[]>(`/lyrics/search?${params}`, signal)
}

export async function fetchLyricsFromProvider(
  provider: string,
  songId: string,
): Promise<Lyrics> {
  return get<Lyrics>(`/lyrics/fetch/${provider}/${songId}`)
}
