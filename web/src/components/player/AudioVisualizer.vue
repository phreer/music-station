<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { AudioAnalysis } from '@/audio/analysis'
import { SpectrumRenderer } from '@/audio/renderer'

const props = defineProps<{
  analysis: AudioAnalysis
  playing: boolean
  suspended: boolean
  dark: boolean
}>()
const canvas = ref<HTMLCanvasElement | null>(null)
let renderer: SpectrumRenderer | null = null
let resizeObserver: ResizeObserver | null = null
let intersectionObserver: IntersectionObserver | null = null
let visible = false
let width = 0
let height = 0
let sampleRate = 0
const mobile = window.matchMedia('(max-width: 700px)')

function update() {
  renderer?.update(visible && width > 0 && height > 0, props.playing, props.suspended)
}

function resize() {
  if (!canvas.value || !renderer) return
  try {
    const style = getComputedStyle(canvas.value)
    sampleRate = props.analysis.sampleRate
    renderer.resize(width, height, style.getPropertyValue('--app-primary').trim(), style.getPropertyValue('--app-text').trim())
    update()
  } catch {
    renderer.stop()
    props.analysis.fail()
  }
}

watch(() => [props.playing, props.suspended], update, { flush: 'sync' })
watch(() => props.suspended, () => {
  if (sampleRate !== props.analysis.sampleRate) resize()
})
watch(() => props.dark, resize, { flush: 'post' })

onMounted(() => {
  if (!canvas.value) return
  try {
    renderer = new SpectrumRenderer(canvas.value, props.analysis)
    resizeObserver = new ResizeObserver(([entry]) => {
      if (!entry) return
      width = entry.contentRect.width
      height = entry.contentRect.height
      resize()
    })
    resizeObserver.observe(canvas.value)
    intersectionObserver = new IntersectionObserver(([entry]) => {
      visible = entry?.isIntersecting ?? false
      update()
    })
    intersectionObserver.observe(canvas.value)
    mobile.addEventListener('change', resize)
  } catch {
    props.analysis.fail()
  }
})

onBeforeUnmount(() => {
  renderer?.stop()
  resizeObserver?.disconnect()
  intersectionObserver?.disconnect()
  mobile.removeEventListener('change', resize)
})
</script>

<template>
  <canvas ref="canvas" :class="$style.canvas" aria-hidden="true" />
</template>

<style module>
.canvas { display: block; width: 100%; height: 100%; }
</style>
