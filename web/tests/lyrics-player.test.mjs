import assert from 'node:assert/strict'
import { test } from 'node:test'
import { createServer } from 'vite'
import { createPinia, setActivePinia } from 'pinia'

const server = await createServer({
  optimizeDeps: { noDiscovery: true, include: [] },
  server: { middlewareMode: true, watch: null, ws: false },
})
let reduced = false
const values = new Map()
globalThis.window = { location: { origin: 'http://localhost' }, matchMedia: () => ({ matches: reduced }) }
globalThis.localStorage = { getItem: key => values.get(key) ?? null, setItem: (key, value) => values.set(key, value) }
const originalFetch = globalThis.fetch
const reply = (body, status = 200) => new Response(JSON.stringify(body), { status, headers: { 'Content-Type': 'application/json' } })
const deferred = () => { let resolve; const promise = new Promise(r => { resolve = r }); return { promise, resolve } }
const text = (track_id, content = '[00:01.00]First\n[00:02.00]Second') => ({ track_id, content, format: 'lrc' })

try {
  const { useLyricsStore } = await server.ssrLoadModule('/src/stores/lyrics.ts')
  const { usePlayerStore } = await server.ssrLoadModule('/src/stores/player.ts')
  const { useUiStore } = await server.ssrLoadModule('/src/stores/ui.ts')

  await test('late old responses and old finally cannot replace the current track or loading state', async () => {
    setActivePinia(createPinia())
    const lyrics = useLyricsStore()
    const first = deferred()
    const second = deferred()
    let count = 0
    const signals = []
    globalThis.fetch = (url, opts) => { signals.push(opts.signal); return ++count === 1 ? first.promise : second.promise }
    const old = lyrics.loadForTrack('old')
    const next = lyrics.loadForTrack('new')
    assert.equal(signals[0].aborted, true)
    first.resolve(reply(text('old')))
    await old
    assert.equal(lyrics.isLoading, true)
    assert.equal(lyrics.currentLyrics, null)
    second.resolve(reply(text('new')))
    await next
    assert.equal(lyrics.currentLyrics.track_id, 'new')
    assert.equal(lyrics.isLoading, false)
  })

  await test('reset aborts pending work and immediately clears word, line, flags and loading state', async () => {
    setActivePinia(createPinia())
    const lyrics = useLyricsStore()
    globalThis.fetch = async () => reply(text('old'))
    await lyrics.loadForTrack('old')
    lyrics.updateCurrentTime(2)
    lyrics.currentWordIndex = 4
    const pending = deferred()
    let signal
    globalThis.fetch = (url, options) => { signal = options.signal; return pending.promise }
    const load = lyrics.loadForTrack('new')
    assert.equal(lyrics.currentWordIndex, -1)
    assert.equal(lyrics.hasLyrics, false)
    assert.equal(lyrics.currentLineIndex, -1)
    lyrics.reset()
    assert.equal(signal.aborted, true)
    assert.equal(lyrics.isLoading, false)
    pending.resolve(reply(text('new')))
    await load
    assert.equal(lyrics.currentLyrics, null)
    assert.equal(lyrics.hasLyrics, false)
    assert.equal(lyrics.loadError, false)
    assert.deepEqual(lyrics.parsedLines, [])
  })

  await test('newer response remains current when an aborted old request resolves later; errors are distinct from no lyrics', async () => {
    setActivePinia(createPinia())
    const lyrics = useLyricsStore()
    const oldResponse = deferred()
    globalThis.fetch = () => oldResponse.promise
    const old = lyrics.loadForTrack('old')
    globalThis.fetch = async () => reply(text('new'))
    await lyrics.loadForTrack('new')
    oldResponse.resolve(reply(text('old')))
    await old
    assert.equal(lyrics.currentLyrics.track_id, 'new')
    lyrics.updateCurrentTime(2.1)
    assert.equal(lyrics.currentLineIndex, 1)
    lyrics.updateCurrentTime(1.1)
    assert.equal(lyrics.currentLineIndex, 0)
    globalThis.fetch = async () => reply(null, 404)
    await lyrics.loadForTrack('missing')
    assert.equal(lyrics.hasLyrics, false)
    assert.equal(lyrics.loadError, false)
    globalThis.fetch = async () => reply(null, 503)
    await lyrics.loadForTrack('error')
    assert.equal(lyrics.loadError, true)
    assert.equal(lyrics.isLoading, false)
  })

  await test('reduced motion suppresses saved visualization for this load but permits a deliberate enable', () => {
    values.set('player-visualizer', 'true')
    reduced = true
    setActivePinia(createPinia())
    const ui = useUiStore()
    assert.equal(ui.visualizerEnabled, false)
    assert.equal(ui.immersiveVisible, false)
    assert.equal(values.get('player-visualizer'), 'true')
    ui.setVisualizerEnabled(true)
    assert.equal(ui.visualizerEnabled, true)
    ui.setVisualizerEnabled(false, false)
    assert.equal(values.get('player-visualizer'), 'true')
    reduced = false
    setActivePinia(createPinia())
    assert.equal(useUiStore().visualizerEnabled, true)
    useUiStore().setVisualizerEnabled(false)
    setActivePinia(createPinia())
    assert.equal(useUiStore().visualizerEnabled, false)
  })

  await test('player distinguishes actual playback and waiting, syncs seeks immediately and removes listeners on teardown', async () => {
    setActivePinia(createPinia())
    const player = usePlayerStore()
    class Audio extends EventTarget {
      currentTime = 0
      duration = 100
      paused = true
      ended = false
      readyState = 4
      play() { this.paused = false; this.dispatchEvent(new Event('play')); return Promise.resolve() }
      pause() { this.paused = true; this.dispatchEvent(new Event('pause')) }
      load() { this.dispatchEvent(new Event('emptied')) }
      removeAttribute() {}
    }
    const audio = new Audio()
    let preparations = 0
    player.initAudio(audio, () => preparations++)
    globalThis.fetch = async () => reply(null)
    await player.playTrack('song')
    assert.equal(preparations, 1)
    assert.equal(player.isPlaying, true)
    assert.equal(player.isActuallyPlaying, false)
    audio.dispatchEvent(new Event('playing'))
    assert.equal(player.isActuallyPlaying, true)
    audio.dispatchEvent(new Event('waiting'))
    assert.equal(player.isActuallyPlaying, false)
    assert.equal(player.isWaiting, true)
    audio.dispatchEvent(new Event('playing'))
    assert.equal(player.isWaiting, false)
    audio.dispatchEvent(new Event('loadedmetadata'))
    player.seek(75)
    assert.equal(audio.currentTime, 75)
    assert.equal(player.currentTime, 75)
    audio.currentTime = 12
    audio.dispatchEvent(new Event('seeked'))
    assert.equal(player.currentTime, 12)
    player.stop()
    assert.equal(player.currentTrackId, null)
    assert.equal(player.isActuallyPlaying, false)
    player.destroyAudio()
    audio.dispatchEvent(new Event('playing'))
    assert.equal(player.isPlaying, false)
    assert.equal(player.isActuallyPlaying, false)
  })

  await test('superseded play rejection cannot pause the newer song or increment the old play count', async () => {
    setActivePinia(createPinia())
    const player = usePlayerStore()
    let rejectFirst
    const interrupted = new Promise((resolve, reject) => { rejectFirst = reject })
    let plays = 0
    const audio = new EventTarget()
    audio.play = () => ++plays === 1 ? interrupted : Promise.resolve()
    audio.pause = () => {}
    player.initAudio(audio)
    const requests = []
    globalThis.fetch = async url => { requests.push(url); return reply(null) }
    const old = player.playTrack('old')
    await player.playTrack('new')
    audio.dispatchEvent(new Event('playing'))
    rejectFirst(new DOMException('Interrupted by new source', 'AbortError'))
    await old
    assert.equal(player.currentTrackId, 'new')
    assert.equal(player.isPlaying, true)
    assert.equal(player.isActuallyPlaying, true)
    assert.equal(requests.length, 1)
    assert.match(requests[0], /\/new\/play$/)
    player.destroyAudio()
  })
} finally {
  globalThis.fetch = originalFetch
  delete globalThis.window
  delete globalThis.localStorage
  await server.close()
}
