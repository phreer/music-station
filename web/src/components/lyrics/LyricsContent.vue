<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { NButton, NEmpty, NSpin } from 'naive-ui'
import { tr } from '@/i18n'
import { useLyricsStore } from '@/stores/lyrics'
import { usePlayerStore } from '@/stores/player'
import { useUiStore } from '@/stores/ui'

defineProps<{ immersive?: boolean }>()

const lyrics = useLyricsStore()
const player = usePlayerStore()
const ui = useUiStore()
const scroller = ref<HTMLElement | null>(null)
const following = ref(true)

async function follow(instant = false) {
  await nextTick()
  const container = scroller.value
  const line = container?.querySelector<HTMLElement>(`[data-idx="${lyrics.currentLineIndex}"]`)
  if (!container || !line || !following.value) return
  // Scroll only this container; scrollIntoView can move the entire application.
  container.scrollTo({
    top: line.offsetTop - container.clientHeight / 2 + line.offsetHeight / 2,
    behavior: instant || window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'instant' : 'smooth',
  })
}

function returnToCurrent() {
  following.value = true
  void follow()
}

function manualScroll() { following.value = false }
function handleKey(event: KeyboardEvent) {
  if (['ArrowUp', 'ArrowDown', 'PageUp', 'PageDown', 'Home', 'End', ' '].includes(event.key)) manualScroll()
}

watch(() => lyrics.currentLineIndex, () => { void follow() })
watch(() => lyrics.parsedLines, () => { void follow(true) }, { immediate: true })
watch(() => player.currentTrackId, () => {
  following.value = true
  if (scroller.value) scroller.value.scrollTop = 0
})
watch(() => ui.lyricsFontSize, () => { void follow(true) })
</script>

<template>
  <div :class="[$style.content, immersive && $style.immersive]" :style="{ '--lyrics-font-size': `${ui.lyricsFontSize}px` }">
    <div ref="scroller" :class="$style.scroller" tabindex="0" :aria-label="tr('Lyrics')"
      :aria-busy="lyrics.isLoading" @wheel.passive="manualScroll" @touchstart.passive="manualScroll"
      @pointerdown="manualScroll" @keydown="handleKey">
      <div v-if="lyrics.isLoading" :class="$style.state" role="status"><NSpin size="small" /><span>{{ tr('Loading lyrics') }}</span></div>
      <NEmpty v-else-if="lyrics.loadError" :description="tr('Failed to load lyrics')" :class="$style.state" />
      <NEmpty v-else-if="!player.currentTrack" :description="tr('Nothing playing')" :class="$style.state" />
      <NEmpty v-else-if="!lyrics.hasLyrics" :description="tr('No lyrics')" :class="$style.state" />
      <div v-else-if="lyrics.currentLyrics?.format === 'plain' || !lyrics.parsedLines.length" :class="$style.plain">{{ lyrics.currentLyrics?.content }}</div>
      <div v-else :class="$style.lines">
        <div v-for="(line, idx) in lyrics.parsedLines" :key="idx" :data-idx="idx"
          :aria-current="idx === lyrics.currentLineIndex ? 'true' : undefined"
          :class="[$style.line, idx === lyrics.currentLineIndex && $style.active]">
          <template v-if="line.words?.length">
            <span v-for="(word, wi) in line.words" :key="wi"
              :class="idx === lyrics.currentLineIndex && wi <= lyrics.currentWordIndex && $style.wordActive">{{ word.text }}</span>
          </template>
          <template v-else>{{ line.text || '\u00a0' }}</template>
        </div>
      </div>
    </div>
    <NButton v-if="!following && lyrics.currentLineIndex >= 0" :class="$style.follow" size="small" round @click="returnToCurrent">{{ tr('Back to current lyric') }}</NButton>
  </div>
</template>

<style module>
.content { position: relative; flex: 1; min-height: 0; display: flex; flex-direction: column; overflow: hidden; }
.scroller { position: relative; flex: 1; min-height: 0; overflow-y: auto; overscroll-behavior: contain; padding: 32px 20px; scrollbar-width: thin; }
.state { padding: 40px 0; display: flex; flex-direction: column; align-items: center; gap: 14px; color: var(--app-text-muted); }
.plain { white-space: pre-wrap; font-size: var(--lyrics-font-size); line-height: 1.8; color: var(--app-text-muted); }
.lines { display: flex; flex-direction: column; gap: 14px; padding-block: 12px 40px; }
.line { font-size: var(--lyrics-font-size); line-height: 1.6; text-align: center; padding: 8px; border-radius: 8px; color: var(--app-text-muted); transition: color 0.2s, background 0.2s; }
.active { color: var(--app-primary); font-weight: 600; background: var(--app-active-bg); box-shadow: inset 2px 0 var(--app-primary); }
.wordActive { color: var(--app-primary); font-weight: 800; }
.follow { position: absolute; bottom: 12px; left: 50%; transform: translateX(-50%); box-shadow: 0 4px 16px var(--app-shadow); }

.immersive .scroller {
  padding: 48px 24px;
  scrollbar-gutter: stable both-edges;
  scrollbar-color: transparent transparent;
  mask-image: linear-gradient(to bottom, transparent, #000 32px, #000 calc(100% - 32px), transparent);
}
.immersive .scroller:hover,
.immersive .scroller:focus-visible { scrollbar-color: var(--app-scrollbar-thumb) transparent; }
.immersive .scroller::-webkit-scrollbar { width: 6px; }
.immersive .scroller::-webkit-scrollbar-track { background: transparent; }
.immersive .scroller::-webkit-scrollbar-thumb { border-radius: 999px; background: transparent; }
.immersive .scroller:hover::-webkit-scrollbar-thumb,
.immersive .scroller:focus-visible::-webkit-scrollbar-thumb { background: var(--app-scrollbar-thumb); }
.immersive .scroller::-webkit-scrollbar-thumb:hover { background: var(--app-scrollbar-thumb-hover); }
.immersive .scroller:focus-visible { outline-offset: -3px; border-radius: 12px; }
.immersive .lines { gap: 6px; }
.immersive .line {
  font-size: calc(var(--lyrics-font-size) + 3px);
  line-height: 1.6;
  padding-block: 5px;
}
.immersive .active { color: var(--app-text); background: transparent; box-shadow: none; }
.immersive .plain { font-size: calc(var(--lyrics-font-size) + 3px); line-height: 1.6; }
.immersive .follow { background: var(--app-surface); border-color: var(--app-border); box-shadow: 0 4px 16px var(--app-shadow); }
@media (hover: none) {
  .immersive .scroller { scrollbar-color: var(--app-scrollbar-thumb) transparent; }
  .immersive .scroller::-webkit-scrollbar-thumb { background: var(--app-scrollbar-thumb); }
}
@media (max-width: 700px) {
  .immersive .scroller { padding: 32px 12px; }
  .immersive .lines { gap: 4px; }
  .immersive .line, .immersive .plain { font-size: calc(var(--lyrics-font-size) + 1px); }
}
</style>
