import type { Track } from '@/types'

export function listeningStats(tracks: readonly Track[]) {
  const rankedTracks = tracks
    .filter(track => track.play_count > 0)
    .sort((a, b) => b.play_count - a.play_count || a.id.localeCompare(b.id))

  return {
    rankedTracks,
    totalPlays: rankedTracks.reduce((total, track) => total + track.play_count, 0),
    playedTracks: rankedTracks.length,
    coverage: tracks.length ? Math.round(rankedTracks.length / tracks.length * 100) : 0,
  }
}
