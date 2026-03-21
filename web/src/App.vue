<script setup lang="ts">
import { onMounted, computed, watch } from 'vue'
import {
  NConfigProvider,
  NMessageProvider,
  NNotificationProvider,
  darkTheme,
  type GlobalThemeOverrides,
} from 'naive-ui'
import AppLayout from '@/components/layout/AppLayout.vue'
import { useUiStore } from '@/stores/ui'
import { useLibraryStore } from '@/stores/library'
import { usePlaylistStore } from '@/stores/playlists'

const ui = useUiStore()
const library = useLibraryStore()
const playlistStore = usePlaylistStore()

const theme = computed(() => (ui.isDarkMode ? darkTheme : null))

const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#0066cc',
    primaryColorHover: '#0077ee',
    primaryColorPressed: '#0052a3',
  },
}

function syncThemeAttribute() {
  document.documentElement.dataset.theme = ui.isDarkMode ? 'dark' : 'light'
}

watch(() => ui.isDarkMode, syncThemeAttribute)

onMounted(() => {
  syncThemeAttribute()
  library.loadTracks()
  playlistStore.loadPlaylists()
})
</script>

<template>
  <NConfigProvider :theme="theme" :theme-overrides="themeOverrides">
    <NMessageProvider>
      <NNotificationProvider>
        <AppLayout />
      </NNotificationProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>

<style>
/* App-level CSS custom properties for dark mode support.
   Naive UI --n-* vars are scoped to its component subtrees;
   these --app-* vars are globally available via :root. */
:root {
  --app-bg: #ffffff;
  --app-text: #1a1a1a;
  --app-surface: #ffffff;
  --app-border: #e0e0e0;
  --app-hover: rgba(0, 0, 0, 0.04);
  --app-active-bg: rgba(0, 102, 204, 0.08);
  --app-placeholder-bg: #f0f0f0;
  --app-danger: #dc2626;
  --app-danger-bg: rgba(220, 38, 38, 0.08);
  --app-success: #18a058;
  --app-error: #d03050;
  --app-shadow: rgba(0, 0, 0, 0.1);
}

[data-theme='dark'] {
  --app-bg: #18181c;
  --app-text: #e0e0e6;
  --app-surface: #1e1e22;
  --app-border: #333338;
  --app-hover: rgba(255, 255, 255, 0.06);
  --app-active-bg: rgba(0, 102, 204, 0.18);
  --app-placeholder-bg: #2a2a2e;
  --app-danger: #f87171;
  --app-danger-bg: rgba(248, 113, 113, 0.12);
  --app-success: #36d399;
  --app-error: #f87171;
  --app-shadow: rgba(0, 0, 0, 0.3);
}

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
</style>
