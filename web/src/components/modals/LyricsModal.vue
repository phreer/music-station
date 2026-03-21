<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  NModal, NCard, NForm, NFormItem, NInput, NSelect, NButton, NSpace, NTabs, NTabPane,
} from 'naive-ui'
import type { Track, Lyrics } from '@/types'
import { uploadLyrics, deleteLyrics, fetchLyrics } from '@/api/lyrics'
import { useLyricsStore } from '@/stores/lyrics'
import { useLibraryStore } from '@/stores/library'
import LyricsSearchModal from './LyricsSearchModal.vue'

const props = defineProps<{
  show: boolean
  track: Track | null
}>()
const emit = defineEmits<{ 'update:show': [value: boolean] }>()

const lyricsStore = useLyricsStore()
const library = useLibraryStore()

const content = ref('')
const format = ref<'plain' | 'lrc' | 'lrc_word'>('lrc')
const language = ref('')
const source = ref('')
const isSaving = ref(false)
const isDeleting = ref(false)
const error = ref<string | null>(null)
const showSearch = ref(false)

const formatOptions = [
  { label: 'LRC (line-synced)', value: 'lrc' },
  { label: 'LRC Word (word-level)', value: 'lrc_word' },
  { label: 'Plain text', value: 'plain' },
]

// Populate existing lyrics when opening
watch(
  () => props.show,
  async (visible) => {
    if (!visible || !props.track) return
    error.value = null
    if (props.track.has_lyrics) {
      try {
        const existing: Lyrics = await fetchLyrics(props.track.id)
        content.value = existing.content
        format.value = existing.format
        language.value = existing.language ?? ''
        source.value = existing.source ?? ''
      } catch {
        content.value = ''
      }
    } else {
      content.value = ''
      format.value = 'lrc'
      language.value = ''
      source.value = ''
    }
  },
)

function close() {
  emit('update:show', false)
}

async function handleSave() {
  if (!props.track) return
  if (!content.value.trim()) {
    error.value = 'Lyrics content is required'
    return
  }
  isSaving.value = true
  error.value = null
  try {
    await uploadLyrics(props.track.id, content.value, format.value, language.value, source.value)
    // Update library flag and reload lyrics store if this is the current track
    library.updateTrackLocally(props.track.id, { has_lyrics: true })
    if (lyricsStore.currentLyrics?.track_id === props.track.id || !lyricsStore.currentLyrics) {
      await lyricsStore.loadForTrack(props.track.id)
    }
    close()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to save lyrics'
  } finally {
    isSaving.value = false
  }
}

async function handleDelete() {
  if (!props.track) return
  isDeleting.value = true
  try {
    await deleteLyrics(props.track.id)
    library.updateTrackLocally(props.track.id, { has_lyrics: false })
    lyricsStore.reset()
    close()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to delete lyrics'
  } finally {
    isDeleting.value = false
  }
}

function handleSearchResult(text: string, fmt: string) {
  content.value = text
  format.value = fmt as 'plain' | 'lrc' | 'lrc_word'
  source.value = 'online'
}
</script>

<template>
  <NModal :show="show" @update:show="emit('update:show', $event)" :mask-closable="false">
    <NCard
      style="width: 680px; max-width: 98vw"
      :title="`Lyrics — ${track?.title ?? ''}`"
      :bordered="false"
      role="dialog"
      aria-modal="true"
    >
      <NTabs type="line" size="small" style="margin-bottom: 12px">
        <NTabPane name="edit" tab="Edit">
          <NForm>
            <NFormItem label="Content" :feedback="error ?? undefined" :validation-status="error ? 'error' : undefined">
              <NInput
                v-model:value="content"
                type="textarea"
                placeholder="Paste lyrics here..."
                :autosize="{ minRows: 14, maxRows: 22 }"
                style="font-family: monospace; font-size: 13px"
              />
            </NFormItem>
            <NSpace>
              <NFormItem label="Format" style="min-width: 200px">
                <NSelect v-model:value="format" :options="formatOptions" />
              </NFormItem>
              <NFormItem label="Language">
                <NInput v-model:value="language" placeholder="e.g. zh, en" style="width: 100px" />
              </NFormItem>
              <NFormItem label="Source">
                <NInput v-model:value="source" placeholder="e.g. netease" style="width: 130px" />
              </NFormItem>
            </NSpace>
          </NForm>
        </NTabPane>
      </NTabs>

      <template #footer>
        <NSpace justify="space-between" align="center">
          <NSpace>
            <NButton
              v-if="track?.has_lyrics"
              type="error"
              quaternary
              size="small"
              :loading="isDeleting"
              @click="handleDelete"
            >
              Delete Lyrics
            </NButton>
            <NButton size="small" @click="showSearch = true">Search Online</NButton>
          </NSpace>
          <NSpace>
            <NButton @click="close">Cancel</NButton>
            <NButton type="primary" :loading="isSaving" @click="handleSave">Save</NButton>
          </NSpace>
        </NSpace>
      </template>
    </NCard>
  </NModal>

  <LyricsSearchModal
    v-model:show="showSearch"
    :track="track"
    @select="handleSearchResult"
  />
</template>
