// Core data types matching the Rust backend API responses

export interface Track {
  id: string
  title: string
  artist: string
  album: string
  album_artist: string
  genre: string
  year: number | null
  track_number: number | null
  disc_number: number | null
  duration: number
  file_size: number
  file_path: string
  format: string
  has_cover: boolean
  has_lyrics: boolean
  play_count: number
  composer: string
  comment: string
  custom_fields: Record<string, string>
}

export interface Album {
  name: string
  artist: string
  track_count: number
  total_duration: number
  tracks: Track[]
}

export interface Artist {
  name: string
  album_count: number
  track_count: number
  albums: Album[]
}

export interface Playlist {
  id: string
  name: string
  description: string
  tracks: string[] // track IDs
  created_at: string
  updated_at: string
}

export interface Lyrics {
  track_id: string
  content: string
  format: LyricFormat
  language: string
  source: string
}

export type LyricFormat = 'plain' | 'lrc' | 'lrc_word'

export interface LyricsSearchResult {
  song_id: string
  title: string
  artist: string
  album: string
  duration: number
  provider: string
}

export interface LibraryStats {
  total_tracks: number
  total_albums: number
  total_artists: number
  total_duration: number
  total_size: number
  total_play_count: number
}

export interface ParsedLyricsLine {
  time: number
  text: string
  words?: ParsedWord[]
}

export interface ParsedWord {
  text: string
  offset: number
  duration: number
}
