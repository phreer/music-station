import assert from 'node:assert/strict'
import { test } from 'node:test'
import { createServer } from 'vite'

const server = await createServer({
  optimizeDeps: { noDiscovery: true, include: [] },
  server: { middlewareMode: true, watch: null, ws: false },
})

try {
  const { listeningStats } = await server.ssrLoadModule('/src/utils/listeningStats.ts')

  await test('ranks only played tracks, with stable ties and without changing library order', () => {
    const tracks = [
      { id: 'c', play_count: 2 },
      { id: 'unplayed', play_count: 0 },
      { id: 'b', play_count: 10 },
      { id: 'a', play_count: 10 },
    ]
    const original = structuredClone(tracks)
    const stats = listeningStats(tracks)
    assert.deepEqual(stats.rankedTracks.map(track => track.id), ['a', 'b', 'c'])
    assert.equal(stats.totalPlays, 22)
    assert.equal(stats.playedTracks, 3)
    assert.equal(stats.coverage, 75)
    assert.deepEqual(tracks, original)
    assert.deepEqual(listeningStats([...tracks].reverse()).rankedTracks, stats.rankedTracks)
  })

  await test('handles an empty library and a library with no plays', () => {
    const empty = { rankedTracks: [], totalPlays: 0, playedTracks: 0, coverage: 0 }
    assert.deepEqual(listeningStats([]), empty)
    assert.deepEqual(listeningStats([{ id: 'a', play_count: 0 }]), empty)
  })

  await test('totals cover the full library, including tracks outside a Top 50 view', () => {
    const tracks = Array.from({ length: 60 }, (_, i) => ({ id: String(i), play_count: i + 1 }))
    const stats = listeningStats(tracks)
    assert.equal(stats.rankedTracks.length, 60)
    assert.equal(stats.totalPlays, 1830)
    assert.equal(stats.coverage, 100)
    tracks[0].play_count = 100
    assert.equal(listeningStats(tracks).rankedTracks[0].id, '0')
  })
} finally {
  await server.close()
}
