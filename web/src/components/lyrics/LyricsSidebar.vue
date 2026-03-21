<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { NButton, NEmpty, NSpin } from 'naive-ui'
import { X, Music2 } from 'lucide-vue-next'
import { useLyricsStore } from '@/stores/lyrics'
import { usePlayerStore } from '@/stores/player'

const lyrics = useLyricsStore()
const player = usePlayerStore()

const listRef = ref<HTMLElement | null>(null)

// Auto-scroll to active line
watch(
  () => lyrics.currentLineIndex,
  async (idx) => {
    if (idx < 0 || !listRef.value) return
    await nextTick()
    const el = listRef.value.querySelector(`[data-idx="${idx}"]`) as HTMLElement | null
    el?.scrollIntoView({ behavior: 'smooth', block: 'center' })
  },
)
</script>

<template>
  <div :class="$style.sidebar">
    <div :class="$style.header">
      <span :class="$style.title">
        <Music2 :size="14" style="margin-right: 6px; vertical-align: middle" />
        Lyrics
      </span>
      <NButton quaternary circle size="tiny" @click="lyrics.toggleSidebar">
        <template #icon><X :size="14" /></template>
      </NButton>
    </div>

    <div :class="$style.body">
      <NSpin v-if="lyrics.isLoading" :class="$style.spinner" />

      <NEmpty
        v-else-if="!lyrics.hasLyrics && player.currentTrack"
        description="No lyrics"
        :class="$style.empty"
      />

      <NEmpty
        v-else-if="!player.currentTrack"
        description="Nothing playing"
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
  border-left: 1px solid var(--n-border-color, #e0e0e0);
  background: var(--n-color, #fff);
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid var(--n-border-color, #e0e0e0);
  flex-shrink: 0;
}

.title {
  font-size: 13px;
  font-weight: 600;
  opacity: 0.7;
  letter-spacing: 0.5px;
  text-transform: uppercase;
}

.body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 10px;
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
  font-size: 14px;
  line-height: 1.8;
  white-space: pre-wrap;
  opacity: 0.8;
  padding: 0 4px;
}

.lineList {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.line {
  font-size: 14px;
  line-height: 1.6;
  text-align: center;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.3s ease;
  opacity: 0.4;
  cursor: default;
}

.linePast {
  opacity: 0.35;
}

.lineActive {
  opacity: 1;
  font-size: 16px;
  font-weight: 600;
  color: var(--n-primary-color, #0066cc);
  background: rgba(0, 102, 204, 0.06);
}

.word {
  transition: color 0.15s, font-weight 0.15s;
}

.wordActive {
  color: var(--n-primary-color, #0066cc);
  font-weight: 700;
}
</style>
