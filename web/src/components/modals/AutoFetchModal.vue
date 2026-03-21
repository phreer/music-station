<script setup lang="ts">
/**
 * AutoFetchModal: Bulk-fetch lyrics for tracks that don't have them yet.
 * Runs sequentially per track, shows live progress.
 */
import { ref, computed } from 'vue'
import { NModal, NCard, NButton, NSpace, NProgress, NScrollbar, NTag } from 'naive-ui'
import { CheckCircle2, XCircle, Loader2 } from 'lucide-vue-next'
import type { Track } from '@/types'
import { searchLyrics, fetchLyricsFromProvider } from '@/api/lyrics'
import { uploadLyrics } from '@/api/lyrics'
import { useLibraryStore } from '@/stores/library'

const props = defineProps<{
  show: boolean
  tracks: Track[] // tracks to process (caller filters to those without lyrics)
}>()
const emit = defineEmits<{ 'update:show': [value: boolean] }>()

const library = useLibraryStore()

type TaskStatus = 'pending' | 'running' | 'done' | 'failed' | 'skipped'
interface TaskResult {
  track: Track
  status: TaskStatus
  message: string
}

const tasks = ref<TaskResult[]>([])
const isRunning = ref(false)
const doneCount = computed(() => tasks.value.filter((t) => t.status !== 'pending' && t.status !== 'running').length)
const progress = computed(() =>
  tasks.value.length === 0 ? 0 : Math.round((doneCount.value / tasks.value.length) * 100),
)
const isFinished = computed(() => isRunning.value === false && doneCount.value === tasks.value.length && tasks.value.length > 0)

function reset() {
  tasks.value = props.tracks.map((t) => ({ track: t, status: 'pending', message: '' }))
}

// Initialize on open
function handleShow(v: boolean) {
  if (v) reset()
  else if (!isRunning.value) emit('update:show', false)
}

async function start() {
  if (isRunning.value) return
  isRunning.value = true

  for (const task of tasks.value) {
    task.status = 'running'
    task.message = 'Searching…'
    try {
      const q = `${task.track.title} ${task.track.artist}`
      // Try NetEase first
      let results = await searchLyrics(q, 'netease').catch(() => [])
      let provider = 'netease'
      if (results.length === 0) {
        results = await searchLyrics(q, 'qqmusic').catch(() => [])
        provider = 'qqmusic'
      }
      if (results.length === 0) {
        task.status = 'skipped'
        task.message = 'No results'
        continue
      }
      const top = results[0]!
      task.message = `Fetching from ${provider}…`
      const lyrics = await fetchLyricsFromProvider(provider, top.song_id)
      await uploadLyrics(task.track.id, lyrics.content, lyrics.format, lyrics.language ?? '', provider)
      library.updateTrackLocally(task.track.id, { has_lyrics: true })
      task.status = 'done'
      task.message = `OK (${provider})`
    } catch (e) {
      task.status = 'failed'
      task.message = e instanceof Error ? e.message : 'Error'
    }
  }

  isRunning.value = false
}

function close() {
  if (!isRunning.value) emit('update:show', false)
}
</script>

<template>
  <NModal :show="show" @update:show="handleShow" :mask-closable="!isRunning">
    <NCard
      style="width: 500px; max-width: 96vw"
      title="Auto-Fetch Lyrics"
      :bordered="false"
      role="dialog"
      aria-modal="true"
    >
      <NSpace vertical :size="14">
        <div v-if="tasks.length === 0" style="opacity: 0.6; font-size: 14px">
          No tracks selected (all already have lyrics, or none provided).
        </div>
        <template v-else>
          <div style="font-size: 14px; opacity: 0.7">
            {{ tasks.length }} tracks to process
          </div>
          <NProgress
            type="line"
            :percentage="progress"
            :indicator-placement="'inside'"
            :processing="isRunning"
          />
          <NScrollbar style="max-height: 300px">
            <div v-for="(task, i) in tasks" :key="i" :class="$style.taskRow">
              <CheckCircle2 v-if="task.status === 'done'" :size="14" :class="$style.iconDone" />
              <XCircle v-else-if="task.status === 'failed' || task.status === 'skipped'" :size="14" :class="$style.iconFail" />
              <Loader2 v-else-if="task.status === 'running'" :size="14" :class="$style.iconSpin" />
              <div v-else :class="$style.iconPending" />
              <span :class="$style.taskTitle">{{ task.track.title }}</span>
              <NTag v-if="task.status !== 'pending'" size="tiny" :type="task.status === 'done' ? 'success' : task.status === 'running' ? 'info' : 'warning'">
                {{ task.message }}
              </NTag>
            </div>
          </NScrollbar>
        </template>
      </NSpace>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="close" :disabled="isRunning">
            {{ isFinished ? 'Close' : 'Cancel' }}
          </NButton>
          <NButton
            type="primary"
            :loading="isRunning"
            :disabled="tasks.length === 0 || isFinished"
            @click="start"
          >
            {{ isFinished ? 'Done' : 'Start' }}
          </NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>
</template>

<style module>
.taskRow {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px;
  font-size: 13px;
  border-radius: 4px;
}

.taskTitle {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.iconDone { color: #18a058; flex-shrink: 0; }
.iconFail { color: #d03050; flex-shrink: 0; }
.iconSpin {
  flex-shrink: 0;
  animation: spin 1s linear infinite;
}
.iconPending {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 2px solid var(--n-border-color, #ccc);
  flex-shrink: 0;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
