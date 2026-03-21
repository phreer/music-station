import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Lyrics, ParsedLyricsLine } from '@/types'
import { fetchLyrics } from '@/api/lyrics'
import { useLrcParser } from '@/composables/useLrcParser'

export const useLyricsStore = defineStore('lyrics', () => {
  const currentLyrics = ref<Lyrics | null>(null)
  const isLoading = ref(false)
  const parsedLines = ref<ParsedLyricsLine[]>([])
  const currentLineIndex = ref(-1)
  // Index of the last active word within the current line (for word-level highlighting)
  const currentWordIndex = ref(-1)
  const sidebarVisible = ref(true)
  const hasLyrics = ref(false)
  let loadController: AbortController | null = null

  const { parseLrc, parseWordLevel } = useLrcParser()

  const hasParsedLines = computed(() => parsedLines.value.length > 0)

  async function loadForTrack(trackId: string) {
    loadController?.abort()
    loadController = new AbortController()
    isLoading.value = true
    currentLyrics.value = null
    parsedLines.value = []
    currentLineIndex.value = -1
    try {
      const lyrics = await fetchLyrics(trackId, loadController.signal)
      currentLyrics.value = lyrics
      hasLyrics.value = true
      parseContent(lyrics)
    } catch (e) {
      if (e instanceof DOMException && e.name === 'AbortError') return
      hasLyrics.value = false
      currentLyrics.value = null
    } finally {
      isLoading.value = false
    }
  }

  function parseContent(lyrics: Lyrics) {
    if (lyrics.format === 'lrc') {
      parsedLines.value = parseLrc(lyrics.content)
    } else if (lyrics.format === 'lrc_word') {
      parsedLines.value = parseWordLevel(lyrics.content)
    } else {
      // Plain text: one "line" per text line, no timestamps
      parsedLines.value = lyrics.content.split('\n').map((text) => ({
        time: -1,
        text,
      }))
    }
  }

  function updateCurrentTime(time: number) {
    const lines = parsedLines.value
    if (lines.length === 0) return

    const prevIdx = currentLineIndex.value

    // Incremental forward scan: during normal playback the active line is
    // almost always the current or next line, so start from prevIdx.
    let idx = -1
    if (prevIdx >= 0 && prevIdx < lines.length && lines[prevIdx]!.time <= time) {
      // Scan forward from the previous position
      idx = prevIdx
      for (let i = prevIdx + 1; i < lines.length; i++) {
        if (lines[i]!.time >= 0 && lines[i]!.time <= time) {
          idx = i
        } else {
          break
        }
      }
    } else {
      // Seek / jump: use binary search to find the last line with time <= current
      let lo = 0
      let hi = lines.length - 1
      while (lo <= hi) {
        const mid = (lo + hi) >>> 1
        if (lines[mid]!.time <= time) {
          idx = mid
          lo = mid + 1
        } else {
          hi = mid - 1
        }
      }
      // Skip lines without timestamps (time < 0)
      if (idx >= 0 && lines[idx]!.time < 0) idx = -1
    }

    if (currentLineIndex.value !== idx) {
      currentLineIndex.value = idx
    }
    // Compute active word index for the current line
    let wIdx = -1
    if (idx >= 0) {
      const words = parsedLines.value[idx]?.words
      if (words && words.length > 0) {
        for (let w = 0; w < words.length; w++) {
          if (time >= words[w]!.offset) {
            wIdx = w
          } else {
            break
          }
        }
      }
    }
    if (currentWordIndex.value !== wIdx) {
      currentWordIndex.value = wIdx
    }
  }

  function reset() {
    currentLyrics.value = null
    parsedLines.value = []
    currentLineIndex.value = -1
    currentWordIndex.value = -1
    hasLyrics.value = false
  }

  function toggleSidebar() {
    sidebarVisible.value = !sidebarVisible.value
  }

  return {
    currentLyrics,
    isLoading,
    parsedLines,
    currentLineIndex,
    currentWordIndex,
    sidebarVisible,
    hasLyrics,
    hasParsedLines,
    loadForTrack,
    updateCurrentTime,
    reset,
    toggleSidebar,
  }
})
