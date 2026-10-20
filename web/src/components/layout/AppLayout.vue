<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import AppHeader from './AppHeader.vue'
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
const playerShell = ref<HTMLElement | null>(null)
const playerHeight = ref(0)
let playerObserver: ResizeObserver | null = null

function updatePlayerHeight() {
  playerHeight.value = player.currentTrack ? (playerShell.value?.getBoundingClientRect().height ?? 0) : 0
}

onMounted(() => {
  playerObserver = new ResizeObserver(updatePlayerHeight)
  if (playerShell.value) playerObserver.observe(playerShell.value)
  updatePlayerHeight()
})

watch(() => player.currentTrack, async () => {
  await nextTick()
  updatePlayerHeight()
})

const showLeftSidebar = computed(
  () => ui.lyricsPanelSide === 'left' && lyrics.sidebarVisible && player.currentTrack && !ui.immersiveVisible,
)
const showRightSidebar = computed(
  () => ui.lyricsPanelSide === 'right' && lyrics.sidebarVisible && player.currentTrack && !ui.immersiveVisible,
)

let activePointerId: number | null = null
const isResizing = ref(false)
let resizeStartX = 0
let resizeStartWidth = 0
let resizeDirection = 1

function startResize(event: PointerEvent) {
  if (event.button !== 0 || activePointerId !== null) return
  event.preventDefault()
  const handle = event.currentTarget as HTMLElement
  activePointerId = event.pointerId
  isResizing.value = true
  resizeStartX = event.clientX
  resizeStartWidth = ui.sidebarWidth
  resizeDirection = ui.lyricsPanelSide === 'left' ? 1 : -1
  handle.setPointerCapture(event.pointerId)
  window.addEventListener('pointermove', handleResize)
  window.addEventListener('pointerup', stopResize)
  window.addEventListener('pointercancel', stopResize)
}

function handleResize(event: PointerEvent) {
  if (activePointerId !== event.pointerId) return

  ui.setSidebarWidth(resizeStartWidth + (event.clientX - resizeStartX) * resizeDirection)
}

function stopResize(event?: PointerEvent) {
  if (event && activePointerId !== event.pointerId) return
  activePointerId = null
  isResizing.value = false
  window.removeEventListener('pointermove', handleResize)
  window.removeEventListener('pointerup', stopResize)
  window.removeEventListener('pointercancel', stopResize)
}

onBeforeUnmount(() => {
  stopResize()
  playerObserver?.disconnect()
})
</script>

<template>
  <div :inert="ui.immersiveVisible || undefined" :class="[$style.layout, isResizing && $style.resizing]" :style="{ '--app-player-height': `${playerHeight}px` }">
    <AppHeader />
    <div :class="$style.body">
      <Transition name="sidebar">
        <div
          v-if="showLeftSidebar"
          :class="[$style.lyricsSidebarShell, $style.lyricsSidebarLeft]"
          :style="{ width: ui.sidebarWidth + 'px' }"
        >
          <div :class="[$style.resizeHandle, $style.resizeHandleLeft]" @pointerdown="startResize" />
          <div :class="$style.lyricsSidebar">
            <LyricsSidebar v-if="!ui.immersiveVisible" />
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
            <LyricsSidebar v-if="!ui.immersiveVisible" />
          </div>
          <div :class="[$style.resizeHandle, $style.resizeHandleRight]" @pointerdown="startResize" />
        </div>
      </Transition>
    </div>
    <div v-show="player.currentTrack" ref="playerShell" :class="$style.playerShell">
      <MusicPlayer />
    </div>
    <QueuePanel />
    <QueueToggle />
  </div>
</template>

<style module>
.layout {
  display: flex;
  flex-direction: column;
  height: 100dvh;
  overflow: hidden;
}

.body {
  flex: 1;
  min-height: 0;
  display: flex;
  overflow: hidden;
  position: relative;
}

.main {
  flex: 1;
  min-height: 0;
  min-width: 0;
  overflow-y: auto;
  scrollbar-width: none; /* Firefox */
}

.main::-webkit-scrollbar {
  display: none; /* Chrome/Safari/Edge */
}

.playerShell {
  flex-shrink: 0;
  padding: 0 20px calc(16px + env(safe-area-inset-bottom));
}

@media (max-width: 600px) {
  .playerShell { padding-inline: 8px; }
}

.lyricsSidebarShell {
  flex-shrink: 0;
  display: flex;
  overflow: hidden;
  position: relative;
}

.lyricsSidebar {
  flex: 1;
  min-width: 0;
}

.lyricsSidebarLeft {
  padding: 24px 16px 24px 20px;
  order: -1;
}

.lyricsSidebarRight {
  padding: 24px 20px 24px 16px;
  order: 1;
}

.resizeHandle {
  position: absolute;
  top: 24px;
  bottom: 24px;
  z-index: 1;
  width: 16px;
  cursor: col-resize;
  touch-action: none;
}

.resizeHandle::before {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  width: 3px;
  height: 40px;
  transform: translate(-50%, -50%);
  border-radius: 999px;
  background: var(--app-text-muted);
  opacity: 0.4;
  transition: height 0.2s ease, opacity 0.2s ease, background-color 0.2s ease;
}

.resizeHandle:hover::before,
.resizing .resizeHandle::before {
  height: 56px;
  opacity: 1;
  background: var(--app-primary);
}

.resizeHandleLeft {
  right: 8px;
}

.resizeHandleRight {
  left: 8px;
}

.resizing,
.resizing * {
  cursor: col-resize !important;
  user-select: none;
}

@media (max-width: 700px) {
  .lyricsSidebarShell {
    position: absolute;
    inset: 0 0 0 auto;
    z-index: 20;
    width: min(380px, 100%) !important;
    max-width: 100%;
    padding: 8px;
    background: var(--app-bg);
    box-shadow: -8px 0 24px var(--app-shadow);
  }
  .lyricsSidebarLeft { right: auto; left: 0; }
  .resizeHandle { display: none; }
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
