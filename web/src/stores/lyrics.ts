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

  const { parseLrc, parseWordLevel } = useLrcParser()

  const hasParsedLines = computed(() => parsedLines.value.length > 0)

  async function loadForTrack(trackId: string) {
    isLoading.value = true
    currentLyrics.value = null
    parsedLines.value = []
    currentLineIndex.value = -1
    try {
      const lyrics = await fetchLyrics(trackId)
      currentLyrics.value = lyrics
      hasLyrics.value = true
      parseContent(lyrics)
    } catch {
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
    if (parsedLines.value.length === 0) return
    // Find the last line whose timestamp <= current time
    let idx = -1
    for (let i = 0; i < parsedLines.value.length; i++) {
      const line = parsedLines.value[i]!
      if (line.time >= 0 && line.time <= time) {
        idx = i
      } else if (line.time > time) {
        break
      }
    }
    // Only write reactive refs when values actually change
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
