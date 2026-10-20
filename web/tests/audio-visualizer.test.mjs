import assert from 'node:assert/strict'
import { test } from 'node:test'
import { createServer } from 'vite'

const server = await createServer({
  optimizeDeps: { noDiscovery: true, include: [] },
  server: { middlewareMode: true, watch: null, ws: false },
})
const originals = Object.fromEntries(['window', 'performance', 'requestAnimationFrame', 'cancelAnimationFrame'].map(key => [key, globalThis[key]]))
globalThis.window = { devicePixelRatio: 2, matchMedia: () => ({ matches: false }) }
let clock = 0
let serial = 0
let pending = new Map()
globalThis.performance = { now: () => clock }
globalThis.requestAnimationFrame = callback => { pending.set(++serial, callback); return serial }
globalThis.cancelAnimationFrame = id => pending.delete(id)
const step = time => {
  clock = time
  const callbacks = [...pending.values()]
  pending.clear()
  for (const callback of callbacks) callback(time)
}

class Graph extends EventTarget {
  state = 'suspended'
  sampleRate = 44100
  destination = {}
  connections = new Set()
  sources = 0
  analysers = 0
  resumes = 0
  reads = 0
  closed = 0
  readError = false
  resumeError = false
  source = {
    connect: node => this.connections.add(node),
    disconnect: node => node ? this.connections.delete(node) : this.connections.clear(),
  }
  analyser = {
    frequencyBinCount: 512,
    getByteFrequencyData: data => {
      this.reads++
      if (this.readError) throw new Error('Analysis failed')
      data.fill(0)
    },
    disconnect() {},
  }
  createMediaElementSource() { this.sources++; return this.source }
  createAnalyser() { this.analysers++; return this.analyser }
  async resume() {
    this.resumes++
    if (this.resumeError) throw new Error('Resume denied')
    this.state = 'running'
    this.dispatchEvent(new Event('statechange'))
  }
  async close() { this.closed++; this.state = 'closed' }
}
const makeCanvas = () => {
  const ctx = {
    setTransform() {}, createLinearGradient: () => ({ addColorStop() {} }),
    clearRect() {}, fillRect() {},
  }
  return { width: 0, height: 0, getContext: () => ctx, ctx }
}

try {
  const { AudioAnalysis } = await server.ssrLoadModule('/src/audio/analysis.ts')
  const { SpectrumRenderer } = await server.ssrLoadModule('/src/audio/renderer.ts')
  const { frequencyBands, bandAmplitude, canvasScale, RenderBudget } = await server.ssrLoadModule('/src/audio/spectrum.ts')
  const setup = () => {
    const graph = new Graph()
    let errors = 0
    let changes = 0
    const analysis = new AudioAnalysis(() => changes++, () => errors++, () => graph)
    return { graph, analysis, errors: () => errors, changes: () => changes }
  }

  await test('lazy graph is created once and analysis switches preserve direct audio output', async () => {
    const { graph, analysis } = setup()
    await analysis.prepare({}, false)
    assert.equal(graph.sources, 0)
    await analysis.prepare({}, true)
    const first = { stop() { analysis.stop(this) } }
    for (let i = 0; i < 20; i++) {
      await analysis.prepare({}, true)
      assert.equal(analysis.start(first), true)
      const buffer = analysis.read(first)
      assert.equal(analysis.read(first), buffer)
      analysis.stop(first)
      assert.deepEqual([...graph.connections], [graph.destination])
    }
    assert.equal(graph.sources, 1)
    assert.equal(graph.analysers, 1)
    assert.equal(graph.resumes, 1)
    assert.equal(graph.analyser.fftSize, 1024)
    assert.equal(graph.analyser.smoothingTimeConstant, 0.75)
    analysis.destroy()
    analysis.destroy()
    assert.equal(graph.connections.size, 0)
    assert.equal(graph.closed, 1)
    const changes = graph.resumes
    await analysis.prepare({}, true)
    assert.equal(graph.resumes, changes)
  })

  await test('resume failure reports once, stops analysis and permits explicit audio resume without rebuilding', async () => {
    const { graph, analysis, errors } = setup()
    await analysis.prepare({}, true)
    graph.state = 'suspended'
    graph.resumeError = true
    await analysis.prepare({}, false)
    assert.equal(analysis.ready, false)
    assert.equal(errors(), 1)
    assert.deepEqual([...graph.connections], [graph.destination])
    graph.resumeError = false
    await analysis.prepare({}, false)
    assert.equal(graph.state, 'running')
    assert.equal(analysis.ready, false)
    assert.equal(graph.sources, 1)
    assert.equal(errors(), 1)
    analysis.destroy()
  })

  await test('rejected first resume leaves native playback untouched and does not retry graph creation', async () => {
    const { graph, analysis, errors } = setup()
    graph.resumeError = true
    await analysis.prepare({}, true)
    assert.equal(graph.sources, 0)
    assert.equal(graph.connections.size, 0)
    assert.equal(errors(), 1)
    graph.resumeError = false
    await analysis.prepare({}, true)
    assert.equal(graph.sources, 0)
    assert.equal(analysis.ready, false)
    analysis.destroy()
  })

  await test('pending resumes keep native output, create only one source, and cannot reroute after teardown', async () => {
    for (const dispose of [false, true]) {
      const { graph, analysis } = setup()
      let resume
      const pending = new Promise(resolve => { resume = resolve })
      graph.resume = () => pending
      const first = analysis.prepare({}, true)
      const second = analysis.prepare({}, true)
      assert.equal(graph.sources, 0)
      if (dispose) analysis.destroy()
      graph.state = 'running'
      resume()
      await Promise.all([first, second])
      assert.equal(graph.sources, dispose ? 0 : 1)
      if (!dispose) analysis.destroy()
    }
  })

  await test('unsupported audio context is attempted once, with no automatic retry', async () => {
    let attempts = 0
    let errors = 0
    const analysis = new AudioAnalysis(() => {}, () => errors++, () => { attempts++; throw new Error('Unsupported') })
    for (let i = 0; i < 20; i++) await analysis.prepare({}, true)
    assert.equal(attempts, 1)
    assert.equal(errors, 1)
    analysis.destroy()
  })

  await test('log bands are valid at low and high sample rates, including repeated low-frequency bins', () => {
    for (const rate of [1000, 8000, 22050, 44100, 48000, 96000, 192000]) {
      for (const count of [24, 48]) {
        const bands = frequencyBands(rate, 512, count)
        const silence = new Uint8Array(512)
        const full = new Uint8Array(512).fill(255)
        for (let i = 0; i < count; i++) {
          assert.ok(bands[i * 2] >= 0)
          assert.ok(bands[i * 2 + 1] <= 512)
          assert.ok(bands[i * 2 + 1] > bands[i * 2])
          assert.equal(bandAmplitude(silence, bands, i), 0)
          assert.equal(bandAmplitude(full, bands, i), 1)
        }
      }
    }
    for (const [width, height] of [[1000, 80], [4000, 2000]]) {
      const scale = canvasScale(width, height, 3)
      assert.ok(scale <= 1.5)
      assert.ok(width * height * scale * scale <= 1_000_001)
    }
  })

  await test('one loop, 30 FPS limit, stable silence, pause decay and immediate hidden/buffering stop', async () => {
    pending.clear()
    const { graph, analysis } = setup()
    await analysis.prepare({}, true)
    const canvas = makeCanvas()
    let bars = 0
    canvas.ctx.fillRect = () => bars++
    const renderer = new SpectrumRenderer(canvas, analysis)
    renderer.resize(800, 80, '#4062d8', '#fff')
    for (let i = 0; i < 20; i++) renderer.update(true, true)
    assert.equal(pending.size, 1)
    for (let time = 0; time <= 1000; time += 1000 / 60) step(time)
    assert.ok(graph.reads >= 29 && graph.reads <= 31, `reads: ${graph.reads}`)
    assert.equal(bars, 0)
    renderer.update(true, false)
    assert.deepEqual([...graph.connections], [graph.destination])
    const reads = graph.reads
    for (let time = 1020; time <= 1240; time += 20) step(time)
    assert.equal(pending.size, 0)
    assert.equal(graph.reads, reads)
    renderer.update(true, true)
    renderer.update(false, true)
    assert.equal(pending.size, 0)
    renderer.update(true, true)
    renderer.update(true, true, true)
    assert.equal(pending.size, 0)
    analysis.destroy()
  })

  await test('switching renderers revokes old loops and interruption cancels drawing without disconnecting sound', async () => {
    pending.clear()
    const { graph, analysis } = setup()
    await analysis.prepare({}, true)
    let old = new SpectrumRenderer(makeCanvas(), analysis)
    old.resize(800, 80, '#000', '#fff')
    old.update(true, true)
    for (let i = 0; i < 20; i++) {
      const next = new SpectrumRenderer(makeCanvas(), analysis)
      next.resize(800, 80, '#000', '#fff')
      next.update(true, true)
      old.stop()
      assert.equal(pending.size, 1)
      assert.equal(graph.connections.size, 2)
      old = next
    }
    graph.state = 'interrupted'
    graph.dispatchEvent(new Event('statechange'))
    assert.equal(pending.size, 0)
    assert.deepEqual([...graph.connections], [graph.destination])
    analysis.destroy()
  })

  await test('analysis and drawing exceptions cancel RAF, preserve sound and show only one warning', async () => {
    for (const kind of ['analysis', 'drawing']) {
      pending.clear()
      const { graph, analysis, errors } = setup()
      await analysis.prepare({}, true)
      const canvas = makeCanvas()
      if (kind === 'analysis') graph.readError = true
      else canvas.ctx.clearRect = () => { throw new Error('Canvas lost') }
      const renderer = new SpectrumRenderer(canvas, analysis)
      renderer.resize(800, 80, '#000', '#fff')
      renderer.update(true, true)
      assert.doesNotThrow(() => step(0))
      assert.equal(pending.size, 0)
      assert.deepEqual([...graph.connections], [graph.destination])
      assert.equal(errors(), 1)
      analysis.fail()
      assert.equal(errors(), 1)
      analysis.destroy()
    }
  })

  await test('budget requires two consecutive slow P95 groups and never upgrades until reset', () => {
    const budget = new RenderBudget()
    const group = cost => { for (let i = 0; i < 60; i++) budget.record(cost) }
    group(3)
    group(1)
    group(3)
    assert.equal(budget.fps, 30)
    group(3)
    assert.equal(budget.fps, 20)
    assert.equal(budget.p95, 3)
    group(0.1)
    assert.equal(budget.fps, 20)
    budget.reset()
    assert.equal(budget.fps, 30)
  })

  await test('slow draws downgrade the running renderer to 20 FPS and DPR 1 across view switches', async () => {
    pending.clear()
    const { graph, analysis } = setup()
    await analysis.prepare({}, true)
    const canvas = makeCanvas()
    canvas.ctx.clearRect = () => { clock += 3 }
    const renderer = new SpectrumRenderer(canvas, analysis)
    renderer.resize(800, 80, '#000', '#fff')
    assert.equal(canvas.width, 1200)
    renderer.update(true, true)
    for (let t = 0; t < 4100; t += 1000 / 60) step(t)
    assert.equal(analysis.budget.degraded, true)
    assert.equal(canvas.width, 800)
    renderer.stop()
    const next = new SpectrumRenderer(makeCanvas(), analysis)
    next.resize(800, 80, '#000', '#fff')
    next.update(true, true)
    const reads = graph.reads
    for (let t = 4200; t < 5200; t += 1000 / 60) step(t)
    assert.ok(graph.reads - reads >= 19 && graph.reads - reads <= 21)
    assert.equal(analysis.budget.degraded, true)
    analysis.destroy()
  })
} finally {
  for (const [key, value] of Object.entries(originals)) {
    if (value === undefined) delete globalThis[key]
    else globalThis[key] = value
  }
  await server.close()
}
