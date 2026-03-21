<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  NModal, NCard, NForm, NFormItem, NInput, NButton, NSpace, NGi, NGrid,
} from 'naive-ui'
import type { Track } from '@/types'
import { updateTrack } from '@/api/tracks'
import { useLibraryStore } from '@/stores/library'

const props = defineProps<{
  show: boolean
  track: Track | null
}>()
const emit = defineEmits<{ 'update:show': [value: boolean] }>()

const library = useLibraryStore()

// Editable fields
const title = ref('')
const artist = ref('')
const album = ref('')
const albumArtist = ref('')
const genre = ref('')
const year = ref<string | null>(null)
const trackNumber = ref<string | null>(null)
const discNumber = ref<string | null>(null)
const composer = ref('')
const comment = ref('')

const isSaving = ref(false)
const error = ref<string | null>(null)

watch(
  () => props.show,
  (visible) => {
    if (!visible || !props.track) return
    const t = props.track
    title.value = t.title ?? ''
    artist.value = t.artist ?? ''
    album.value = t.album ?? ''
    albumArtist.value = t.album_artist ?? ''
    genre.value = t.genre ?? ''
    year.value = t.year
    trackNumber.value = t.track_number
    discNumber.value = t.disc_number
    composer.value = t.composer ?? ''
    comment.value = t.comment ?? ''
    error.value = null
  },
)

function close() {
  emit('update:show', false)
}

async function handleSave() {
  if (!props.track) return
  isSaving.value = true
  error.value = null
  try {
    const updated = await updateTrack(props.track.id, {
      title: title.value,
      artist: artist.value,
      album: album.value,
      album_artist: albumArtist.value,
      genre: genre.value,
      year: year.value,
      track_number: trackNumber.value,
      disc_number: discNumber.value,
      composer: composer.value,
      comment: comment.value,
    })
    library.updateTrackLocally(props.track.id, updated)
    close()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to save'
  } finally {
    isSaving.value = false
  }
}
</script>

<template>
  <NModal :show="show" @update:show="emit('update:show', $event)" :mask-closable="false">
    <NCard
      style="width: 600px; max-width: 98vw"
      title="Edit Track Metadata"
      :bordered="false"
      role="dialog"
      aria-modal="true"
    >
      <NForm label-placement="top" label-width="auto">
        <NGrid :x-gap="14" :cols="2">
          <NGi :span="2">
            <NFormItem label="Title">
              <NInput v-model:value="title" placeholder="Title" />
            </NFormItem>
          </NGi>
          <NGi>
            <NFormItem label="Artist">
              <NInput v-model:value="artist" placeholder="Artist" />
            </NFormItem>
          </NGi>
          <NGi>
            <NFormItem label="Album Artist">
              <NInput v-model:value="albumArtist" placeholder="Album Artist" />
            </NFormItem>
          </NGi>
          <NGi :span="2">
            <NFormItem label="Album">
              <NInput v-model:value="album" placeholder="Album" />
            </NFormItem>
          </NGi>
          <NGi>
            <NFormItem label="Genre">
              <NInput v-model:value="genre" placeholder="Genre" />
            </NFormItem>
          </NGi>
          <NGi>
            <NFormItem label="Year">
              <NInput v-model:value="year" placeholder="Year" />
            </NFormItem>
          </NGi>
          <NGi>
            <NFormItem label="Track #">
              <NInput v-model:value="trackNumber" placeholder="Track number" />
            </NFormItem>
          </NGi>
          <NGi>
            <NFormItem label="Disc #">
              <NInput v-model:value="discNumber" placeholder="Disc number" />
            </NFormItem>
          </NGi>
          <NGi>
            <NFormItem label="Composer">
              <NInput v-model:value="composer" placeholder="Composer" />
            </NFormItem>
          </NGi>
          <NGi>
            <!-- spacer -->
          </NGi>
          <NGi :span="2">
            <NFormItem label="Comment">
              <NInput
                v-model:value="comment"
                type="textarea"
                placeholder="Comment"
                :autosize="{ minRows: 2, maxRows: 4 }"
              />
            </NFormItem>
          </NGi>
        </NGrid>
      </NForm>

      <div v-if="error" style="color: var(--n-error-color, #d03050); font-size: 13px; margin-top: 4px">
        {{ error }}
      </div>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="close">Cancel</NButton>
          <NButton type="primary" :loading="isSaving" @click="handleSave">Save</NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>
</template>
