import assert from 'node:assert/strict'
import { test } from 'node:test'
import { createServer } from 'vite'
import { computed } from 'vue'

// Use Vite's existing TypeScript loader so tests exercise the same modules as the app.
const server = await createServer({
  optimizeDeps: { noDiscovery: true, include: [] },
  server: { middlewareMode: true, watch: null, ws: false },
})
const values = new Map()
globalThis.localStorage = {
  getItem: (key) => values.get(key) ?? null,
  setItem: (key, value) => values.set(key, value),
}
globalThis.document = { documentElement: { lang: 'en' } }

async function freshModule(savedLanguage) {
  values.clear()
  if (savedLanguage !== undefined) values.set('language', savedLanguage)
  server.moduleGraph.invalidateAll()
  return server.ssrLoadModule('/src/i18n/index.ts')
}

try {
  await test('switches both directions, updates reactive labels and persists the selection', async () => {
    const { language, setLanguage, tr } = await freshModule()
    const label = computed(() => tr('Tracks'))
    assert.equal(language.value, 'en')
    assert.equal(label.value, 'Tracks')
    setLanguage('zh')
    assert.equal(label.value, '歌曲')
    assert.equal(values.get('language'), 'zh')
    assert.equal(document.documentElement.lang, 'zh-CN')
    assert.equal(tr('{count} tracks', { count: 3 }), '3 首歌曲')
    setLanguage('en')
    assert.equal(label.value, 'Tracks')
    assert.equal(values.get('language'), 'en')
    assert.equal(document.documentElement.lang, 'en')
  })

  await test('restores a saved language and falls back for invalid preferences', async () => {
    assert.equal((await freshModule('zh')).tr('Settings'), '设置')
    assert.equal((await freshModule('en')).tr('Settings'), 'Settings')
    const i18n = await freshModule('invalid')
    assert.equal(i18n.language.value, 'en')
    i18n.setLanguage('invalid')
    assert.equal(i18n.language.value, 'en')
  })

  await test('handles unavailable storage and preserves unknown messages and interpolation values', async () => {
    const storage = globalThis.localStorage
    globalThis.localStorage = {
      getItem() { throw new Error('Blocked') },
      setItem() { throw new Error('Blocked') },
    }
    try {
      const { language, setLanguage, tr } = await freshModule()
      assert.equal(language.value, 'en')
      setLanguage('zh')
      assert.equal(tr('Settings'), '设置')
      assert.equal(tr('Server error'), 'Server error')
      assert.equal(tr('Play {title}', { title: 'A {count} <song>' }), '播放 A {count} <song>')
      assert.equal(tr('toString'), 'toString')
    } finally {
      globalThis.localStorage = storage
    }
  })

  await test('Chinese messages preserve every English interpolation placeholder', async () => {
    const { zh } = await server.ssrLoadModule('/src/i18n/zh.ts')
    const placeholders = (text) => [...text.matchAll(/\{\w+\}/g)].map(([value]) => value).sort()
    for (const [english, chinese] of Object.entries(zh)) {
      assert.deepEqual(placeholders(chinese), placeholders(english), english)
    }
  })
} finally {
  await server.close()
  delete globalThis.localStorage
  delete globalThis.document
}
