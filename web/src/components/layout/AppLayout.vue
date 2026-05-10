<script setup lang="ts">
import { computed, onBeforeUnmount } from 'vue'
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

const showLeftSidebar = computed(
  () => ui.lyricsPanelSide === 'left' && lyrics.sidebarVisible && player.currentTrack,
)
const showRightSidebar = computed(
  () => ui.lyricsPanelSide === 'right' && lyrics.sidebarVisible && player.currentTrack,
)

let activePointerId: number | null = null

function startResize(event: PointerEvent) {
  activePointerId = event.pointerId
  window.addEventListener('pointermove', handleResize)
  window.addEventListener('pointerup', stopResize)
  window.addEventListener('pointercancel', stopResize)
}

function handleResize(event: PointerEvent) {
  if (activePointerId !== event.pointerId) return

  if (ui.lyricsPanelSide === 'left') {
    ui.setSidebarWidth(event.clientX)
    return
  }

  ui.setSidebarWidth(window.innerWidth - event.clientX)
}

function stopResize(event?: PointerEvent) {
  if (event && activePointerId !== event.pointerId) return
  activePointerId = null
  window.removeEventListener('pointermove', handleResize)
  window.removeEventListener('pointerup', stopResize)
  window.removeEventListener('pointercancel', stopResize)
}

onBeforeUnmount(() => {
  stopResize()
})
</script>

<template>
  <div :class="$style.layout">
    <AppHeader />
    <AppNav />
    <div :class="$style.body">
      <Transition name="sidebar">
        <div
          v-if="showLeftSidebar"
          :class="[$style.lyricsSidebarShell, $style.lyricsSidebarLeft]"
          :style="{ width: ui.sidebarWidth + 'px' }"
        >
          <div :class="[$style.resizeHandle, $style.resizeHandleLeft]" @pointerdown="startResize" />
          <div :class="$style.lyricsSidebar">
            <LyricsSidebar />
          </div>
        </div>
      </Transition>
      <main :class="$style.main">
        <RouterView v-slot="{ Component }">
          <KeepAlive :max="5">
            <component :is="Component" />
          </KeepAlive>
        </RouterView>
      </main>
      <Transition name="sidebar">
        <div
          v-if="showRightSidebar"
          :class="[$style.lyricsSidebarShell, $style.lyricsSidebarRight]"
          :style="{ width: ui.sidebarWidth + 'px' }"
        >
          <div :class="$style.lyricsSidebar">
            <LyricsSidebar />
          </div>
          <div :class="[$style.resizeHandle, $style.resizeHandleRight]" @pointerdown="startResize" />
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

.lyricsSidebarShell {
  flex-shrink: 0;
  display: flex;
  overflow: hidden;
  padding-bottom: 80px; /* align with player bar */
  position: relative;
}

.lyricsSidebar {
  flex: 1;
  min-width: 0;
}

.lyricsSidebarLeft {
  order: -1;
}

.lyricsSidebarRight {
  order: 1;
}

.resizeHandle {
  width: 10px;
  flex-shrink: 0;
  align-self: stretch;
  cursor: col-resize;
  position: relative;
  touch-action: none;
}

.resizeHandle::before {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  left: 50%;
  width: 2px;
  transform: translateX(-50%);
  border-radius: 999px;
  background: var(--app-border);
  opacity: 0.55;
  transition: opacity 0.2s ease, background-color 0.2s ease;
}

.resizeHandle:hover::before {
  opacity: 1;
  background: var(--n-primary-color, #0066cc);
}

.resizeHandleLeft {
  order: 1;
}

.resizeHandleRight {
  order: -1;
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
