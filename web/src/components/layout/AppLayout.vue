<script setup lang="ts">
import AppHeader from './AppHeader.vue'
import AppNav from './AppNav.vue'
import MusicPlayer from '@/components/player/MusicPlayer.vue'
import QueuePanel from '@/components/queue/QueuePanel.vue'
import QueueToggle from '@/components/queue/QueueToggle.vue'
import LyricsSidebar from '@/components/lyrics/LyricsSidebar.vue'
import TracksView from '@/views/TracksView.vue'
import AlbumsView from '@/views/AlbumsView.vue'
import ArtistsView from '@/views/ArtistsView.vue'
import PlaylistsView from '@/views/PlaylistsView.vue'
import StatsView from '@/views/StatsView.vue'
import { useUiStore } from '@/stores/ui'
import { useLyricsStore } from '@/stores/lyrics'
import { usePlayerStore } from '@/stores/player'

const ui = useUiStore()
const lyrics = useLyricsStore()
const player = usePlayerStore()
</script>

<template>
  <div :class="$style.layout">
    <AppHeader />
    <AppNav />
    <div :class="$style.body">
      <main :class="$style.main">
        <TracksView v-if="ui.currentView === 'tracks'" />
        <AlbumsView v-else-if="ui.currentView === 'albums'" />
        <ArtistsView v-else-if="ui.currentView === 'artists'" />
        <PlaylistsView v-else-if="ui.currentView === 'playlists'" />
        <StatsView v-else-if="ui.currentView === 'stats'" />
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
