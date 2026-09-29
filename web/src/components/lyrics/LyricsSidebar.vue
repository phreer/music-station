<script setup lang="ts">
import { tr } from '@/i18n'
import { ref, watch, nextTick } from 'vue'
import { NButton, NEmpty, NSpin } from 'naive-ui'
import { Minus, RotateCcw, Plus, X, Music2, PanelLeft, PanelRight } from 'lucide-vue-next'
import { useLyricsStore } from '@/stores/lyrics'
import { usePlayerStore } from '@/stores/player'
import { useUiStore } from '@/stores/ui'

const lyrics = useLyricsStore()
const player = usePlayerStore()
const ui = useUiStore()

const listRef = ref<HTMLElement | null>(null)

// Auto-scroll to active line
watch(
  () => lyrics.currentLineIndex,
  async (idx) => {
    if (idx < 0 || !listRef.value) return
    await nextTick()
    const el = listRef.value.querySelector(`[data-idx="${idx}"]`) as HTMLElement | null
    el?.scrollIntoView({ behavior: window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'instant' : 'smooth', block: 'center' })
  },
)
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

    <div
      :class="$style.body"
      :style="{
        '--lyrics-font-size': `${ui.lyricsFontSize}px`,
        '--lyrics-active-font-size': `${ui.lyricsFontSize}px`,
      }"
    >
      <NSpin v-if="lyrics.isLoading" :class="$style.spinner" />

      <NEmpty
        v-else-if="!lyrics.hasLyrics && player.currentTrack"
        :description="tr('No lyrics')"
        :class="$style.empty"
      />

      <NEmpty
        v-else-if="!player.currentTrack"
        :description="tr('Nothing playing')"
        :class="$style.empty"
      />

      <!-- Plain text lyrics (no timestamps) -->
      <div
        v-else-if="lyrics.currentLyrics?.format === 'plain'"
        :class="$style.plainText"
      >
        {{ lyrics.currentLyrics.content }}
      </div>

      <!-- Synced LRC lyrics -->
      <div v-else-if="lyrics.parsedLines.length > 0" ref="listRef" :class="$style.lineList">
        <div
          v-for="(line, idx) in lyrics.parsedLines"
          :key="idx"
          :data-idx="idx"
          :class="[
            $style.line,
            idx === lyrics.currentLineIndex && $style.lineActive,
            idx < lyrics.currentLineIndex && $style.linePast,
          ]"
        >
          <!-- Word-level highlighting for lrc_word format -->
          <template v-if="line.words && line.words.length > 0">
            <span
              v-for="(word, wi) in line.words"
              :key="wi"
              :class="[
                $style.word,
                idx === lyrics.currentLineIndex &&
                  wi <= lyrics.currentWordIndex &&
                  $style.wordActive,
              ]"
            >{{ word.text }}</span>
          </template>
          <template v-else>{{ line.text }}</template>
        </div>
      </div>

      <!-- Fallback: lyrics exist but parsing produced no lines -->
      <div
        v-else-if="lyrics.hasLyrics && lyrics.currentLyrics"
        :class="$style.plainText"
      >
        {{ lyrics.currentLyrics.content }}
      </div>
    </div>
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

.body {
  flex: 1;
  overflow-y: auto;
  padding: 32px 20px;
  scroll-behavior: smooth;
}

.spinner {
  display: flex;
  justify-content: center;
  padding: 40px;
}

.empty {
  padding: 40px 0;
}

.plainText {
  font-size: var(--lyrics-font-size, 17px);
  line-height: 1.8;
  white-space: pre-wrap;
  opacity: 0.8;
  padding: 0 4px;
}

.lineList {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.line {
  font-size: var(--lyrics-font-size, 17px);
  line-height: 1.6;
  text-align: center;
  padding: 4px 8px;
  border-radius: 8px;
  transition: color 0.2s ease, background 0.2s ease;
  color: var(--app-text-muted);
  cursor: default;
}

.linePast {
  color: var(--app-text-muted);
}

.lineActive {
  font-size: var(--lyrics-active-font-size, 19px);
  font-weight: 600;
  color: var(--app-primary);
  background: var(--app-active-bg);
  box-shadow: inset 2px 0 var(--app-primary);
}

.word {
  transition: color 0.15s, font-weight 0.15s;
}

.wordActive {
  color: var(--app-primary);
  font-weight: 700;
}
</style>
