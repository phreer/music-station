import { defineStore } from 'pinia'
import { ref } from 'vue'

export type ViewName = 'tracks' | 'albums' | 'artists' | 'playlists' | 'stats'
export type LyricsPanelSide = 'left' | 'right'

const DEFAULT_LYRICS_FONT_SIZE = 17
const MIN_LYRICS_FONT_SIZE = 13
const MAX_LYRICS_FONT_SIZE = 26
const DEFAULT_LYRICS_PANEL_SIDE: LyricsPanelSide = 'right'
const DEFAULT_LYRICS_PANEL_WIDTH = 380
const MIN_LYRICS_PANEL_WIDTH = 280
const MAX_LYRICS_PANEL_WIDTH = 560

export const useUiStore = defineStore('ui', () => {
  const isDarkMode = ref(localStorage.getItem('theme') === 'dark')
  const sidebarWidth = ref(
    clampLyricsPanelWidth(
      parseInt(localStorage.getItem('lyrics-sidebar-width') ?? String(DEFAULT_LYRICS_PANEL_WIDTH)),
    ),
  )
  const lyricsPanelSide = ref(parseLyricsPanelSide(localStorage.getItem('lyrics-panel-side')))
  const lyricsFontSize = ref(
    clampLyricsFontSize(parseInt(localStorage.getItem('lyrics-font-size') ?? String(DEFAULT_LYRICS_FONT_SIZE))),
  )

  function toggleTheme() {
    isDarkMode.value = !isDarkMode.value
    localStorage.setItem('theme', isDarkMode.value ? 'dark' : 'light')
  }

  function setSidebarWidth(width: number) {
    sidebarWidth.value = clampLyricsPanelWidth(width)
    localStorage.setItem('lyrics-sidebar-width', String(sidebarWidth.value))
  }

  function setLyricsPanelSide(side: LyricsPanelSide) {
    lyricsPanelSide.value = side
    localStorage.setItem('lyrics-panel-side', side)
  }

  function toggleLyricsPanelSide() {
    setLyricsPanelSide(lyricsPanelSide.value === 'left' ? 'right' : 'left')
  }

  function setLyricsFontSize(size: number) {
    lyricsFontSize.value = clampLyricsFontSize(size)
    localStorage.setItem('lyrics-font-size', String(lyricsFontSize.value))
  }

  function increaseLyricsFontSize() {
    setLyricsFontSize(lyricsFontSize.value + 1)
  }

  function decreaseLyricsFontSize() {
    setLyricsFontSize(lyricsFontSize.value - 1)
  }

  function resetLyricsFontSize() {
    setLyricsFontSize(DEFAULT_LYRICS_FONT_SIZE)
  }

  return {
    isDarkMode,
    sidebarWidth,
    lyricsPanelSide,
    lyricsFontSize,
    defaultLyricsFontSize: DEFAULT_LYRICS_FONT_SIZE,
    defaultLyricsPanelSide: DEFAULT_LYRICS_PANEL_SIDE,
    defaultSidebarWidth: DEFAULT_LYRICS_PANEL_WIDTH,
    minLyricsFontSize: MIN_LYRICS_FONT_SIZE,
    maxLyricsFontSize: MAX_LYRICS_FONT_SIZE,
    minSidebarWidth: MIN_LYRICS_PANEL_WIDTH,
    maxSidebarWidth: MAX_LYRICS_PANEL_WIDTH,
    toggleTheme,
    setSidebarWidth,
    setLyricsPanelSide,
    toggleLyricsPanelSide,
    setLyricsFontSize,
    increaseLyricsFontSize,
    decreaseLyricsFontSize,
    resetLyricsFontSize,
  }
})

function clampLyricsFontSize(size: number) {
  if (Number.isNaN(size)) return DEFAULT_LYRICS_FONT_SIZE
  return Math.min(MAX_LYRICS_FONT_SIZE, Math.max(MIN_LYRICS_FONT_SIZE, size))
}

function clampLyricsPanelWidth(width: number) {
  if (Number.isNaN(width)) return DEFAULT_LYRICS_PANEL_WIDTH
  return Math.min(MAX_LYRICS_PANEL_WIDTH, Math.max(MIN_LYRICS_PANEL_WIDTH, width))
}

function parseLyricsPanelSide(side: string | null): LyricsPanelSide {
  return side === 'left' || side === 'right' ? side : DEFAULT_LYRICS_PANEL_SIDE
}
