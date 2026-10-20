<script setup lang="ts">
import { computed, ref } from 'vue'
import { NButton, NEmpty } from 'naive-ui'
import { Music, Play, Trophy, Volume2 } from 'lucide-vue-next'
import { tr } from '@/i18n'
import type { Track } from '@/types'
import { coverUrl } from '@/api/client'
import { usePlayerStore } from '@/stores/player'
import { useQueueStore } from '@/stores/queue'

const props = defineProps<{ tracks: Track[]; disabled?: boolean }>()
const player = usePlayerStore()
const queue = useQueueStore()
const limit = ref(10)
const visibleTracks = computed(() => props.tracks.slice(0, limit.value))
const maxPlays = computed(() => props.tracks[0]?.play_count ?? 1)

function playFrom(index: number) {
  const ids = visibleTracks.value.map(track => track.id)
  const id = ids[index]
  if (!id || props.disabled) return
  queue.setQueue(ids, index)
  void player.playTrack(id)
}
</script>

<template>
  <section :class="$style.panel" aria-labelledby="play-ranking-title">
    <div :class="$style.header">
      <div>
        <h2 id="play-ranking-title"><Trophy :size="19" />{{ tr('Most played tracks') }}</h2>
        <p>{{ tr('All-time plays · Tracks in your current library') }}</p>
      </div>
      <div :class="$style.actions">
        <div :class="$style.limits" role="group" :aria-label="tr('Ranking size')">
          <button v-for="size in [10, 25, 50]" :key="size" type="button"
            :aria-pressed="limit === size" @click="limit = size">Top {{ size }}</button>
        </div>
        <NButton size="small" secondary type="primary" :disabled="disabled || !visibleTracks.length" @click="playFrom(0)">
          <template #icon><Play :size="14" /></template>
          {{ tr('Play ranking') }}
        </NButton>
      </div>
    </div>

    <slot>
      <div v-if="!tracks.length" :class="$style.empty">
        <NEmpty :description="tr('No plays yet')">
          <template #icon><Music /></template>
          <template #extra>{{ tr('Play a track to start building your ranking.') }}</template>
        </NEmpty>
      </div>
      <template v-else>
        <div :class="$style.columns" aria-hidden="true">
          <span>#</span><span>{{ tr('Tracks') }}</span><span>{{ tr('Plays') }}</span><span />
        </div>
        <ol :class="$style.list">
          <li v-for="(track, index) in visibleTracks" :key="track.id"
            :class="[$style.row, track.id === player.currentTrackId && $style.current]">
            <span :class="[$style.rank, index < 3 && $style.topRank]">{{ String(index + 1).padStart(2, '0') }}</span>
            <div :class="$style.track">
              <div :class="$style.cover">
                <Music :size="20" aria-hidden="true" />
                <img v-if="track.has_cover" :src="coverUrl(track.id)" alt="" loading="lazy" />
              </div>
              <div :class="$style.metadata">
                <span :class="$style.title" :title="track.title ?? tr('Unknown Title')">{{ track.title ?? tr('Unknown Title') }}</span>
                <span :class="$style.subtitle" :title="[track.artist ?? tr('Unknown Artist'), track.album].filter(Boolean).join(' · ')">
                  {{ track.artist ?? tr('Unknown Artist') }}<template v-if="track.album"> · {{ track.album }}</template>
                </span>
              </div>
            </div>
            <div :class="$style.plays">
              <strong :aria-label="tr('{count} plays', { count: track.play_count.toLocaleString() })">{{ track.play_count.toLocaleString() }}</strong>
              <div :class="$style.bar" aria-hidden="true"><span :style="{ width: `${track.play_count / maxPlays * 100}%` }" /></div>
            </div>
            <NButton quaternary circle size="small" :disabled="disabled"
              :aria-label="tr('Play {title}', { title: track.title ?? tr('Unknown Title') })" @click="playFrom(index)">
              <template #icon>
                <Volume2 v-if="track.id === player.currentTrackId && player.isActuallyPlaying" :size="16" />
                <Play v-else :size="16" />
              </template>
            </NButton>
          </li>
        </ol>
        <p :class="$style.footer">{{ tr('Showing {count} of {total} played tracks', { count: visibleTracks.length, total: tracks.length.toLocaleString() }) }}</p>
      </template>
    </slot>
  </section>
</template>

<style module>
.panel { background: var(--app-surface); border: 1px solid var(--app-border); border-radius: 16px; overflow: hidden; }
.header { display: flex; align-items: center; justify-content: space-between; gap: 20px; padding: 24px; flex-wrap: wrap; }
.header h2 { display: flex; align-items: center; gap: 10px; font-size: 18px; font-weight: 600; }
.header h2 svg { color: var(--app-primary); }
.header p, .footer { color: var(--app-text-muted); font-size: 12px; margin-top: 6px; }
.actions { display: flex; align-items: center; flex-wrap: wrap; gap: 12px; }
.limits { display: flex; padding: 3px; border-radius: 8px; background: var(--app-inset); }
.limits button { border: 0; border-radius: 6px; padding: 6px 10px; font: inherit; font-size: 12px; color: var(--app-text-muted); background: transparent; cursor: pointer; }
.limits button[aria-pressed='true'] { background: var(--app-surface-raised); color: var(--app-primary); box-shadow: 0 1px 4px var(--app-shadow); }
.columns, .row { display: grid; grid-template-columns: 34px minmax(0, 1fr) minmax(80px, 160px) 32px; align-items: center; gap: 16px; padding: 12px 24px; }
.columns { border-block: 1px solid var(--app-border); color: var(--app-text-muted); font-size: 11px; }
.columns > :nth-child(3) { text-align: right; }
.list { list-style: none; }
.row + .row { border-top: 1px solid var(--app-border); }
.row:hover { background: var(--app-hover); }
.current { background: var(--app-active-bg); }
.rank { font-size: 14px; font-variant-numeric: tabular-nums; color: var(--app-text-muted); }
.topRank { color: var(--app-primary); font-weight: 700; }
.track { display: flex; align-items: center; gap: 14px; min-width: 0; }
.cover { position: relative; flex: 0 0 42px; height: 42px; display: grid; place-items: center; border-radius: 8px; overflow: hidden; color: var(--app-text-muted); background: var(--app-placeholder-bg); }
.cover img { position: absolute; width: 100%; height: 100%; object-fit: cover; }
.metadata { min-width: 0; }
.title, .subtitle { display: block; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.title { font-size: 14px; font-weight: 500; }
.subtitle { margin-top: 3px; font-size: 12px; color: var(--app-text-muted); }
.plays { text-align: right; font-variant-numeric: tabular-nums; }
.plays strong { font-size: 14px; font-weight: 600; }
.bar { height: 4px; background: var(--app-inset); border-radius: 8px; margin-top: 7px; overflow: hidden; }
.bar span { display: block; height: 100%; min-width: 2px; background: var(--app-primary); border-radius: inherit; opacity: .65; }
.footer { padding: 16px 24px; margin: 0; border-top: 1px solid var(--app-border); }
.empty { padding: 48px 24px; text-align: center; }
@media (max-width: 600px) {
  .header { padding: 20px 16px; gap: 16px; }
  .actions { width: 100%; justify-content: space-between; gap: 8px; }
  .columns, .row { grid-template-columns: 24px minmax(0, 1fr) 56px 28px; gap: 8px; padding: 12px; }
  .track { gap: 10px; }
  .cover { flex-basis: 34px; height: 34px; }
  .title { font-size: 13px; }
  .subtitle { font-size: 11px; }
  .footer { padding: 16px; }
}
</style>
