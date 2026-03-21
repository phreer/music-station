<script setup lang="ts">
import { ref } from 'vue'
import { NModal, NCard, NForm, NFormItem, NInput, NButton, NSpace } from 'naive-ui'
import { usePlaylistStore } from '@/stores/playlists'

defineProps<{ show: boolean }>()
const emit = defineEmits<{
  'update:show': [value: boolean]
}>()

const playlists = usePlaylistStore()

const name = ref('')
const description = ref('')
const isSubmitting = ref(false)
const error = ref<string | null>(null)

function close() {
  name.value = ''
  description.value = ''
  error.value = null
  emit('update:show', false)
}

async function handleSubmit() {
  if (!name.value.trim()) {
    error.value = 'Name is required'
    return
  }
  isSubmitting.value = true
  error.value = null
  try {
    await playlists.createPlaylist(name.value.trim(), description.value.trim() || undefined)
    close()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to create playlist'
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <NModal :show="show" @update:show="emit('update:show', $event)" :mask-closable="true">
    <NCard
      style="width: 420px; max-width: 95vw"
      title="New Playlist"
      :bordered="false"
      role="dialog"
      aria-modal="true"
    >
      <NForm @submit.prevent="handleSubmit">
        <NFormItem
          label="Name"
          :feedback="error ?? undefined"
          :validation-status="error ? 'error' : undefined"
        >
          <NInput
            v-model:value="name"
            placeholder="Playlist name"
            :autofocus="true"
            @keydown.enter="handleSubmit"
          />
        </NFormItem>
        <NFormItem label="Description (optional)">
          <NInput
            v-model:value="description"
            type="textarea"
            placeholder="Optional description"
            :autosize="{ minRows: 2, maxRows: 4 }"
          />
        </NFormItem>
      </NForm>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="close">Cancel</NButton>
          <NButton type="primary" :loading="isSubmitting" @click="handleSubmit">Create</NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>
</template>
