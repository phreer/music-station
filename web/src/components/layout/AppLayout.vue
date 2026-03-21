<script setup lang="ts">
import AppHeader from './AppHeader.vue'
import AppNav from './AppNav.vue'
import MusicPlayer from '@/components/player/MusicPlayer.vue'
import QueuePanel from '@/components/queue/QueuePanel.vue'
import QueueToggle from '@/components/queue/QueueToggle.vue'
import LyricsSidebar from '@/components/lyrics/LyricsSidebar.vue'
import { useLyricsStore } from '@/stores/lyrics'
import { usePlayerStore } from '@/stores/player'
import { useUiStore } from '@/stores/ui'

const lyrics = useLyricsStore()
const player = usePlayerStore()
const ui = useUiStore()
</script>

<template>
  <div :class="$style.layout">
    <AppHeader />
    <AppNav />
    <div :class="$style.body">
      <main :class="$style.main">
        <RouterView v-slot="{ Component }">
          <KeepAlive :max="5">
            <component :is="Component" />
          </KeepAlive>
        </RouterView>
      </main>
      <!-- Lyrics sidebar: shown when something is playing and sidebar is toggled on -->
      <Transition name="sidebar">
        <div
          v-if="lyrics.sidebarVisible && player.currentTrack"
          :class="$style.lyricsSidebar"
          :style="{ width: ui.sidebarWidth + 'px' }"
        >
          <LyricsSidebar />
        </div>
      </Transition>
    </div>
    <MusicPlayer />
    <QueuePanel />
    <QueueToggle />
  </div>
</template>

<style module>
.layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.main {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 90px; /* space for player bar */
  scrollbar-width: none; /* Firefox */
}

.main::-webkit-scrollbar {
  display: none; /* Chrome/Safari/Edge */
}

.lyricsSidebar {
  flex-shrink: 0;
  overflow: hidden;
  padding-bottom: 80px; /* align with player bar */
}
</style>

<style>
.sidebar-enter-active,
.sidebar-leave-active {
  transition: width 0.25s ease, opacity 0.2s ease;
}

.sidebar-enter-from,
.sidebar-leave-to {
  width: 0 !important;
  opacity: 0;
}
</style>
