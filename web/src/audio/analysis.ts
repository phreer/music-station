import { RenderBudget } from './spectrum'

export interface AnalysisConsumer {
  stop(): void
}

function createContext(): AudioContext {
  const Context = window.AudioContext ?? (window as typeof window & {
    webkitAudioContext?: typeof AudioContext
  }).webkitAudioContext
  if (!Context) throw new Error('Web Audio unavailable')
  return new Context()
}

/** Owns the audio graph for the lifetime of the player, outside Vue reactivity. */
export class AudioAnalysis {
  private context: AudioContext | null = null
  private source: MediaElementAudioSourceNode | null = null
  private analyser: AnalyserNode | null = null
  private data = new Uint8Array(0)
  private consumer: AnalysisConsumer | null = null
  private attempted = false
  private failed = false
  private disposed = false
  readonly budget = new RenderBudget()

  constructor(
    private readonly changed: () => void = () => {},
    private readonly reportError: () => void = () => {},
    private readonly contextFactory = createContext,
  ) {}

  get ready() {
    return !this.failed && !this.disposed && !!this.source && !!this.analyser && this.context?.state === 'running'
  }

  get sampleRate() { return this.context?.sampleRate ?? 48000 }
  get binCount() { return this.data.length }

  /** Invoke from enable/play gestures. Never retry graph creation after a failure. */
  async prepare(audio: HTMLAudioElement, initialize: boolean) {
    if (this.disposed) return
    try {
      if (initialize && !this.attempted) {
        this.attempted = true
        this.context = this.contextFactory()
        this.context.addEventListener('statechange', this.handleState)
        // Allocate the analysis side before rerouting the media element's sound.
        this.analyser = this.context.createAnalyser()
        this.analyser.fftSize = 1024
        this.analyser.smoothingTimeConstant = 0.75
        this.data = new Uint8Array(this.analyser.frequencyBinCount)
      }
      // Even after an analysis failure, explicit play must be able to resume sound.
      if (this.context && this.context.state !== 'running') await this.context.resume()
      if (this.disposed) return
      // Keep native playback intact if the first resume is denied or stays pending.
      if (!this.failed && !this.source && this.analyser && this.context?.state === 'running') {
        this.source = this.context.createMediaElementSource(audio)
        this.source.connect(this.context.destination)
      }
      this.changed()
    } catch {
      this.fail()
    }
  }

  start(consumer: AnalysisConsumer): boolean {
    if (!this.ready || !this.source || !this.analyser) return false
    if (this.consumer === consumer) return true
    this.consumer?.stop()
    try {
      this.source.connect(this.analyser)
      this.consumer = consumer
      return true
    } catch {
      this.fail()
      return false
    }
  }

  stop(consumer: AnalysisConsumer) {
    if (this.consumer !== consumer) return
    this.consumer = null
    // Never disconnect the direct source -> destination connection here.
    try { if (this.analyser) this.source?.disconnect(this.analyser) } catch { /* Already detached. */ }
  }

  read(consumer: AnalysisConsumer): Uint8Array | null {
    if (this.consumer !== consumer || !this.ready) return null
    try {
      this.analyser!.getByteFrequencyData(this.data)
      return this.data
    } catch {
      this.fail()
      return null
    }
  }

  fail() {
    this.consumer?.stop()
    if (!this.failed && !this.disposed) this.reportError()
    this.failed = true
    this.changed()
  }

  private handleState = () => {
    if (!this.ready) this.consumer?.stop()
    this.changed()
  }

  destroy() {
    if (this.disposed) return
    this.consumer?.stop()
    this.disposed = true
    this.context?.removeEventListener('statechange', this.handleState)
    this.source?.disconnect()
    this.analyser?.disconnect()
    void this.context?.close().catch(() => {})
    this.source = null
    this.analyser = null
    this.context = null
    this.data = new Uint8Array(0)
  }
}
