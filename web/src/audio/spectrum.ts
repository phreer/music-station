/** Half-open FFT bin ranges; narrow low-frequency bands may share a bin. */
export function frequencyBands(sampleRate: number, binCount: number, count: number): Uint16Array {
  const bands = new Uint16Array(count * 2)
  const nyquist = sampleRate / 2
  const high = Math.min(16000, nyquist)
  const low = Math.min(60, high)
  for (let i = 0; i < count; i++) {
    const startHz = low * (high / low) ** (i / count)
    const endHz = low * (high / low) ** ((i + 1) / count)
    const start = Math.max(0, Math.min(binCount - 1, Math.floor(startHz / nyquist * binCount)))
    bands[i * 2] = start
    bands[i * 2 + 1] = Math.max(start + 1, Math.min(binCount, Math.ceil(endHz / nyquist * binCount)))
  }
  return bands
}

export function bandAmplitude(data: Uint8Array, bands: Uint16Array, index: number): number {
  const start = bands[index * 2]!
  const end = bands[index * 2 + 1]!
  let sum = 0
  for (let bin = start; bin < end; bin++) sum += data[bin]!
  return sum / ((end - start) * 255)
}

export function canvasScale(width: number, height: number, dpr: number): number {
  return Math.min(dpr, 1.5, Math.sqrt(1_000_000 / Math.max(1, width * height)))
}

/** Shared across view switches, reset only when the user enables visualization. */
export class RenderBudget {
  private samples = new Float64Array(60)
  private count = 0
  private slowGroups = 0
  degraded = false
  p95 = 0

  get fps() { return this.degraded ? 20 : 30 }

  record(duration: number): boolean {
    this.samples[this.count++] = duration
    if (this.count < 60) return false
    this.count = 0
    this.samples.sort()
    this.p95 = this.samples[56]!
    this.slowGroups = this.p95 > 2 ? this.slowGroups + 1 : 0
    if (!this.degraded && this.slowGroups >= 2) {
      this.degraded = true
      return true
    }
    return false
  }

  reset() {
    this.count = 0
    this.slowGroups = 0
    this.degraded = false
    this.p95 = 0
  }
}
