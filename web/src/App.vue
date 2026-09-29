<script setup lang="ts">
import { language } from '@/i18n'
import { onMounted, computed, watch } from 'vue'
import {
  NConfigProvider,
  NMessageProvider,
  NNotificationProvider,
  darkTheme,
  enUS, zhCN, dateEnUS, dateZhCN,
} from 'naive-ui'
import { applyTheme, themeOverrides } from '@/styles/theme'
import '@/styles/tokens.css'
import AppLayout from '@/components/layout/AppLayout.vue'
import { useUiStore } from '@/stores/ui'
import { useLibraryStore } from '@/stores/library'
import { usePlaylistStore } from '@/stores/playlists'

const ui = useUiStore()
const library = useLibraryStore()
const playlistStore = usePlaylistStore()

const theme = computed(() => (ui.isDarkMode ? darkTheme : null))

watch(language, (value) => {
  document.documentElement.lang = value === 'zh' ? 'zh-CN' : 'en'
}, { immediate: true })

const overrides = computed(() => themeOverrides[ui.isDarkMode ? 'dark' : 'light'])

function syncThemeAttribute() {
  applyTheme(ui.isDarkMode ? 'dark' : 'light')
}

syncThemeAttribute()
watch(() => ui.isDarkMode, syncThemeAttribute)

onMounted(() => {
  library.loadTracks()
  playlistStore.loadPlaylists()
})
</script>

<template>
  <NConfigProvider :locale="language === 'zh' ? zhCN : enUS" :date-locale="language === 'zh' ? dateZhCN : dateEnUS" :theme="theme" :theme-overrides="overrides">
    <NMessageProvider>
      <NNotificationProvider>
        <AppLayout />
      </NNotificationProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>

<style>
/* Global reset and base styles */
*,
*::before,
*::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html {
  height: 100%;
}

body {
  height: 100%;
  background-color: var(--app-bg);
  color: var(--app-text);
  font-family:
    -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial,
    sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  transition: background-color 0.2s, color 0.2s;
}

#app {
  height: 100%;
}

/* Keep horizontal track-list scrollbars easy to acquire without making the
   visible thumb dominate the interface. */
.track-scroll-region {
  scrollbar-color: var(--app-scrollbar-thumb) var(--app-scrollbar-track);
}

.track-scroll-region::-webkit-scrollbar {
  height: 14px;
}

.track-scroll-region::-webkit-scrollbar-track {
  background: var(--app-scrollbar-track);
  border-radius: 999px;
}

.track-scroll-region::-webkit-scrollbar-thumb {
  min-width: 48px;
  border: 3px solid transparent;
  border-radius: 999px;
  background: var(--app-scrollbar-thumb);
  background-clip: padding-box;
}

.track-scroll-region::-webkit-scrollbar-thumb:hover {
  background: var(--app-scrollbar-thumb-hover);
  background-clip: padding-box;
}
</style>
