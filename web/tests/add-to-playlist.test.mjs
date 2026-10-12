import assert from 'node:assert/strict'
import { test } from 'node:test'
import { createServer } from 'vite'
import { createPinia, setActivePinia } from 'pinia'

const server = await createServer({
  optimizeDeps: { noDiscovery: true, include: [] },
  server: { middlewareMode: true, watch: null, ws: false },
})
globalThis.window = { location: { origin: 'http://localhost' } }
const originalFetch = globalThis.fetch
const reply = (body, status = 200) => new Response(JSON.stringify(body), {
  status, headers: { 'Content-Type': 'application/json' },
})
const deferred = () => {
  let resolve
  const promise = new Promise((r) => { resolve = r })
  return { promise, resolve }
}
const song = { id: 'song', title: 'Selected song', artist: 'Artist', custom_fields: {} }
const list = (id, tracks = []) => ({ id, name: id, tracks, description: '', created_at: '', updated_at: '' })

try {
  const { usePlaylistStore } = await server.ssrLoadModule('/src/stores/playlists.ts')
  const { useAddToPlaylistStore } = await server.ssrLoadModule('/src/stores/addToPlaylist.ts')
  const setup = () => {
    setActivePinia(createPinia())
    return { playlists: usePlaylistStore(), dialog: useAddToPlaylistStore() }
  }
  const loaded = async (playlists) => {
    while (playlists.isLoading) await new Promise((r) => setImmediate(r))
  }

  await test('opening refreshes playlists and captures a stable song snapshot; missing songs do not open', async () => {
    const { playlists, dialog } = setup()
    let loads = 0
    globalThis.fetch = async () => { loads++; return reply([list('Favorite')]) }
    dialog.open(null)
    assert.equal(dialog.show, false)
    const selected = { ...song, custom_fields: { mood: 'calm' } }
    dialog.open(selected)
    selected.id = 'next'
    selected.title = 'Changed'
    selected.custom_fields.mood = 'loud'
    await loaded(playlists)
    assert.equal(dialog.track.id, 'song')
    assert.equal(dialog.track.title, 'Selected song')
    assert.equal(dialog.track.custom_fields.mood, 'calm')
    dialog.close()
    dialog.open(song)
    await loaded(playlists)
    assert.equal(loads, 2)
  })

  await test('loading failure stays distinguishable from empty playlists and retry recovers', async () => {
    const { playlists, dialog } = setup()
    globalThis.fetch = async () => reply('Unavailable', 503)
    dialog.open(song)
    await loaded(playlists)
    assert.match(playlists.loadError, /503/)
    assert.equal(dialog.show, true)
    assert.equal(await dialog.add('target'), false)
    globalThis.fetch = async () => reply([])
    await playlists.loadPlaylists()
    assert.equal(playlists.loadError, null)
    assert.deepEqual(playlists.playlists, [])
  })

  await test('superseded loads cannot clear the current loading lock or overwrite newer results', async () => {
    const { playlists } = setup()
    const first = deferred()
    const second = deferred()
    let calls = 0
    globalThis.fetch = () => (++calls === 1 ? first.promise : second.promise)
    const oldLoad = playlists.loadPlaylists()
    const newLoad = playlists.loadPlaylists()
    first.resolve(reply([list('old')]))
    await oldLoad
    assert.equal(playlists.isLoading, true)
    second.resolve(reply([list('new')]))
    await newLoad
    assert.equal(playlists.playlists[0].id, 'new')
    assert.equal(playlists.isLoading, false)
  })

  await test('submission blocks duplicate choices, closure and replacement; success updates counts and persists', async () => {
    const { playlists, dialog } = setup()
    const pending = deferred()
    const saved = [list('target'), list('other')]
    const requests = []
    globalThis.fetch = async (url, options) => {
      if (options?.method === 'POST') {
        requests.push(url)
        await pending.promise
        saved[0].tracks.push('song')
        return reply(null)
      }
      return reply(saved)
    }
    dialog.open(song)
    assert.equal(await dialog.add('target'), false)
    await loaded(playlists)
    const reference = playlists.findPlaylist('target')
    const adding = dialog.add('target')
    dialog.close()
    dialog.open({ ...song, id: 'next' })
    assert.equal(dialog.track.id, 'song')
    assert.equal(dialog.show, true)
    assert.equal(await dialog.add('target'), false)
    assert.equal(await dialog.add('other'), false)
    pending.resolve()
    assert.equal(await adding, true)
    assert.equal(requests.length, 1)
    assert.match(requests[0], /\/target\/tracks\/song$/)
    assert.deepEqual(reference.tracks, ['song'])
    assert.equal(dialog.show, false)
    assert.equal(dialog.submittingId, null)
    await playlists.loadPlaylists()
    assert.equal(playlists.findPlaylist('target'), reference)
    assert.equal(reference.tracks.length, 1)
    dialog.open(song)
    await loaded(playlists)
    assert.equal(await dialog.add('target'), false)
    assert.equal(requests.length, 1)
  })

  await test('add failure retains the song and allows retry; creating a playlist does not add automatically', async () => {
    const { playlists, dialog } = setup()
    let fail = true
    let adds = 0
    globalThis.fetch = async (url, options) => {
      if (options?.method === 'POST' && url.endsWith('/playlists')) return reply(list('created'))
      if (options?.method === 'POST') { adds++; return reply(null, fail ? 500 : 200) }
      return reply([list('target')])
    }
    dialog.open(song)
    await loaded(playlists)
    await playlists.createPlaylist('created')
    assert.equal(adds, 0)
    assert.equal(dialog.show, true)
    assert.equal(await dialog.add('created'), false)
    assert.match(dialog.error, /500/)
    assert.equal(dialog.show, true)
    assert.equal(dialog.track.id, 'song')
    assert.equal(dialog.submittingId, null)
    assert.equal(playlists.findPlaylist('created').tracks.length, 0)
    fail = false
    assert.equal(await dialog.add('created'), true)
    assert.equal(playlists.findPlaylist('created').tracks.length, 1)
    assert.equal(dialog.error, null)
  })
} finally {
  globalThis.fetch = originalFetch
  delete globalThis.window
  await server.close()
}
