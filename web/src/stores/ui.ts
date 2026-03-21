import { defineStore } from 'pinia'
import { ref } from 'vue'

export type ViewName = 'tracks' | 'albums' | 'artists' | 'playlists' | 'stats'

export const useUiStore = defineStore('ui', () => {
  const isDarkMode = ref(localStorage.getItem('theme') === 'dark')
  const sidebarWidth = ref(parseInt(localStorage.getItem('lyrics-sidebar-width') ?? '350'))

  function toggleTheme() {
    isDarkMode.value = !isDarkMode.value
    localStorage.setItem('theme', isDarkMode.value ? 'dark' : 'light')
  }

  function setSidebarWidth(width: number) {
    sidebarWidth.value = width
    localStorage.setItem('lyrics-sidebar-width', String(width))
  }

  return {
    isDarkMode,
    sidebarWidth,
    toggleTheme,
    setSidebarWidth,
  }
})
