// Core data types matching the Rust backend API responses

export interface Track {
  id: string
  title: string | null
  artist: string | null
  album: string | null
  album_artist: string | null
  genre: string | null
  year: string | null
  track_number: string | null
  disc_number: string | null
  duration_secs: number | null
  file_size: number
  has_cover: boolean
  has_lyrics: boolean
  play_count: number
  composer: string | null
  comment: string | null
  custom_fields: Record<string, string>
}

export interface Album {
  name: string
  artist: string | null
  track_count: number
  total_duration_secs: number
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
  total_duration_secs: number
  total_size_bytes: number
  total_plays: number
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
