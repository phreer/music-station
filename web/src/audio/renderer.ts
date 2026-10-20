import type { AudioAnalysis } from './analysis'
import { bandAmplitude, canvasScale, frequencyBands } from './spectrum'

/** No reactive writes or layout reads in the animation loop. */
export class SpectrumRenderer {
  private context: CanvasRenderingContext2D
  private raf: number | null = null
  private lastFrame = -Infinity
  private lastMotion = 0
  private decayUntil = 0
  private playing = false
  private width = 0
  private height = 0
  private count = 48
  private bands = new Uint16Array(0)
  private levels = new Float32Array(48)
  private peaks = new Float32Array(48)
  private gradient: CanvasGradient | string = ''
  private primary = ''
  private secondary = ''

  constructor(private canvas: HTMLCanvasElement, private analysis: AudioAnalysis) {
    const context = canvas.getContext('2d')
    if (!context) throw new Error('Canvas unavailable')
    this.context = context
  }

  resize(width: number, height: number, primary: string, secondary: string) {
    this.width = width
    this.height = height
    this.primary = primary
    this.secondary = secondary
    this.count = window.matchMedia('(max-width: 700px)').matches ? 24 : 48
    this.bands = frequencyBands(this.analysis.sampleRate, this.analysis.binCount || 512, this.count)
    const scale = canvasScale(width, height, this.analysis.budget.degraded ? 1 : window.devicePixelRatio || 1)
    this.canvas.width = Math.max(1, Math.floor(width * scale))
    this.canvas.height = Math.max(1, Math.floor(height * scale))
    this.context.setTransform(scale, 0, 0, scale, 0, 0)
    this.gradient = this.context.createLinearGradient(0, height, 0, 0)
    this.gradient.addColorStop(0, primary)
    this.gradient.addColorStop(1, secondary)
  }

  update(visible: boolean, playing: boolean, immediate = false) {
    if (!visible || immediate) { this.stop(); return }
    if (playing) {
      if (!this.analysis.start(this)) { this.stop(); return }
      this.playing = true
      this.decayUntil = 0
      if (this.raf === null) {
        this.lastFrame = -Infinity
        this.lastMotion = performance.now()
        this.raf = requestAnimationFrame(this.frame)
      }
    } else if (this.playing) {
      this.playing = false
      this.analysis.stop(this)
      this.decayUntil = performance.now() + 200
    }
  }

  stop = () => {
    if (this.raf !== null) cancelAnimationFrame(this.raf)
    this.raf = null
    this.playing = false
    this.decayUntil = 0
    this.analysis.stop(this)
    this.levels.fill(0)
    this.peaks.fill(0)
    try { this.context.clearRect(0, 0, this.width, this.height) } catch { /* A lost canvas must still stop. */ }
  }

  private frame = (now: number) => {
    this.raf = null
    if (!this.playing && now >= this.decayUntil) { this.stop(); return }
    if (now - this.lastFrame + 0.1 < 1000 / this.analysis.budget.fps) {
      this.raf = requestAnimationFrame(this.frame)
      return
    }
    const started = performance.now()
    const dt = Math.min(0.1, Math.max(0, (now - this.lastMotion) / 1000))
    this.lastMotion = now
    this.lastFrame = now
    try {
      const sampling = this.playing
      const data = sampling ? this.analysis.read(this) : null
      if (sampling && !data) { this.stop(); return }
      const ctx = this.context
      ctx.clearRect(0, 0, this.width, this.height)
      ctx.fillStyle = this.gradient
      const stride = this.width / this.count
      const barWidth = Math.max(1, stride * 0.65)
      const bass = data ? bandAmplitude(data, this.bands, 0) : 0
      ctx.globalAlpha = 0.78 + bass * 0.16
      for (let i = 0; i < this.count; i++) {
        const target = data ? bandAmplitude(data, this.bands, i) : 0
        const level = this.levels[i]! + (target - this.levels[i]!) * (1 - Math.exp(-dt * (target > this.levels[i]! ? 32 : 8)))
        this.levels[i] = level
        this.peaks[i] = Math.max(level, this.peaks[i]! - dt * 0.35)
        const x = i * stride + (stride - barWidth) / 2
        const h = level * Math.max(0, this.height - 4)
        if (h > 0.2) ctx.fillRect(x, this.height - h, barWidth, h)
        if (this.peaks[i]! > 0.01) ctx.fillRect(x, this.height - this.peaks[i]! * (this.height - 4) - 3, barWidth, 2)
      }
      ctx.globalAlpha = 1
      if (this.analysis.budget.record(performance.now() - started)) {
        this.resize(this.width, this.height, this.primary, this.secondary)
      }
      this.raf = requestAnimationFrame(this.frame)
    } catch {
      this.stop()
      this.analysis.fail()
    }
  }
}
