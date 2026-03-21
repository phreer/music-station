<script setup lang="ts">
import { ref, watch } from 'vue'
import { NModal, NCard, NInput, NButton, NSpace, NSpin, NEmpty } from 'naive-ui'
import { Search } from 'lucide-vue-next'
import type { Track, LyricsSearchResult, Lyrics } from '@/types'
import { searchLyrics, fetchLyricsFromProvider } from '@/api/lyrics'
import { formatDuration } from '@/utils/format'

const props = defineProps<{
  show: boolean
  track: Track | null
}>()
const emit = defineEmits<{
  'update:show': [value: boolean]
  // Emits the fetched content + format back to LyricsModal
  select: [content: string, format: string]
}>()

const query = ref('')
const results = ref<LyricsSearchResult[]>([])
const isSearching = ref(false)
const isFetching = ref<string | null>(null) // song_id being fetched
const error = ref<string | null>(null)

// Pre-fill query from track info when opening
watch(
  () => props.show,
  (visible) => {
    if (visible && props.track) {
      query.value = `${props.track.title} ${props.track.artist}`.trim()
      results.value = []
      error.value = null
    }
  },
)

async function handleSearch() {
  if (!query.value.trim()) return
  isSearching.value = true
  error.value = null
  results.value = []
  try {
    // Try NetEase first, then QQ Music; merge results
    const [netease, qq] = await Promise.allSettled([
      searchLyrics(query.value, 'netease'),
      searchLyrics(query.value, 'qqmusic'),
    ])
    const ne = netease.status === 'fulfilled' ? netease.value : []
    const qqr = qq.status === 'fulfilled' ? qq.value : []
    results.value = [...ne, ...qqr].slice(0, 20)
    if (results.value.length === 0) error.value = 'No results found'
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Search failed'
  } finally {
    isSearching.value = false
  }
}

async function handleSelect(result: LyricsSearchResult) {
  isFetching.value = result.song_id
  try {
    const lyrics: Lyrics = await fetchLyricsFromProvider(result.provider, result.song_id)
    emit('select', lyrics.content, lyrics.format)
    emit('update:show', false)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to fetch lyrics'
  } finally {
    isFetching.value = null
  }
}
</script>

<template>
  <NModal :show="show" @update:show="emit('update:show', $event)" :mask-closable="true">
    <NCard
      style="width: 540px; max-width: 96vw"
      title="Search Lyrics Online"
      :bordered="false"
      role="dialog"
      aria-modal="true"
    >
      <NSpace vertical :size="12">
        <NInput
          v-model:value="query"
          placeholder="Track title + artist…"
          clearable
          @keydown.enter="handleSearch"
        >
          <template #suffix>
            <NButton text :loading="isSearching" @click="handleSearch">
              <template #icon><Search :size="16" /></template>
            </NButton>
          </template>
        </NInput>

        <NSpin :show="isSearching" style="min-height: 80px">
          <NEmpty v-if="!isSearching && results.length === 0 && error" :description="error" style="padding: 20px 0" />
          <NEmpty v-else-if="!isSearching && results.length === 0 && !error" description="Search to find lyrics" style="padding: 20px 0" />

          <div v-else :class="$style.results">
            <div
              v-for="r in results"
              :key="r.provider + r.song_id"
              :class="$style.result"
              @click="handleSelect(r)"
            >
              <NSpin :show="isFetching === r.song_id" :size="'small'">
                <div :class="$style.resultInfo">
                  <span :class="$style.resultTitle">{{ r.title }}</span>
                  <span :class="$style.resultArtist">{{ r.artist }}</span>
                  <span :class="$style.resultAlbum">{{ r.album }}</span>
                </div>
                <div :class="$style.resultMeta">
                  <span :class="$style.provider">{{ r.provider }}</span>
                  <span :class="$style.duration">{{ formatDuration(r.duration) }}</span>
                </div>
              </NSpin>
            </div>
          </div>
        </NSpin>
      </NSpace>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="emit('update:show', false)">Cancel</NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>
</template>

<style module>
.results {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 360px;
  overflow-y: auto;
}

.result {
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.result:hover {
  background: var(--app-hover);
}

.resultInfo {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.resultTitle {
  font-size: 14px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.resultArtist {
  font-size: 12px;
  opacity: 0.6;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.resultAlbum {
  font-size: 11px;
  opacity: 0.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.resultMeta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
  flex-shrink: 0;
}

.provider {
  font-size: 10px;
  opacity: 0.5;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.duration {
  font-size: 11px;
  opacity: 0.5;
  font-variant-numeric: tabular-nums;
}
</style>
