<script setup lang="ts">
import { tr } from '@/i18n'
import { NButton } from 'naive-ui'
import { Minus, RotateCcw, Plus, X, Music2, PanelLeft, PanelRight } from 'lucide-vue-next'
import { useLyricsStore } from '@/stores/lyrics'
import { useUiStore } from '@/stores/ui'
import LyricsContent from './LyricsContent.vue'

const lyrics = useLyricsStore()
const ui = useUiStore()
</script>

<template>
  <div :class="[$style.sidebar, ui.lyricsPanelSide === 'left' && $style.sidebarLeft]">
    <div :class="$style.header">
      <span :class="$style.title">
        <Music2 :size="14" style="margin-right: 6px; vertical-align: middle" />
        {{ tr('Lyrics') }}
      </span>
      <div :class="$style.actions">
        <NButton
          quaternary
          circle
          size="tiny"
          :title="ui.lyricsPanelSide === 'left' ? tr('Move lyrics to right') : tr('Move lyrics to left')"
          @click="ui.toggleLyricsPanelSide"
        >
          <template #icon>
            <PanelRight v-if="ui.lyricsPanelSide === 'left'" :size="14" />
            <PanelLeft v-else :size="14" />
          </template>
        </NButton>
        <NButton
          quaternary
          circle
          size="tiny"
          :title="tr('Smaller lyrics')"
          :disabled="ui.lyricsFontSize <= ui.minLyricsFontSize"
          @click="ui.decreaseLyricsFontSize"
        >
          <template #icon><Minus :size="14" /></template>
        </NButton>
        <NButton
          quaternary
          circle
          size="tiny"
          :title="tr('Reset lyrics size')"
          :disabled="ui.lyricsFontSize === ui.defaultLyricsFontSize"
          @click="ui.resetLyricsFontSize"
        >
          <template #icon><RotateCcw :size="12" /></template>
        </NButton>
        <NButton
          quaternary
          circle
          size="tiny"
          :title="tr('Larger lyrics')"
          :disabled="ui.lyricsFontSize >= ui.maxLyricsFontSize"
          @click="ui.increaseLyricsFontSize"
        >
          <template #icon><Plus :size="14" /></template>
        </NButton>
        <NButton quaternary circle size="tiny" :title="tr('Close lyrics')" @click="lyrics.toggleSidebar">
          <template #icon><X :size="14" /></template>
        </NButton>
      </div>
    </div>

    <LyricsContent />
  </div>
</template>

<style module>
.sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  border: 1px solid var(--app-border);
  border-radius: 14px;
  background: var(--app-surface);
}

.sidebarLeft { border-color: var(--app-border); }

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  border-bottom: 1px solid var(--app-border);
  flex-shrink: 0;
}

.actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.title {
  font-size: 13px;
  font-weight: 600;
  color: var(--app-text-muted);
  letter-spacing: 0;
  text-transform: none;
}

</style>
