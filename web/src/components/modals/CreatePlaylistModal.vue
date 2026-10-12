<script setup lang="ts">
import { tr } from '@/i18n'
import { ref } from 'vue'
import { NModal, NCard, NForm, NFormItem, NInput, NButton, NSpace } from 'naive-ui'
import { usePlaylistStore } from '@/stores/playlists'

defineProps<{ show: boolean }>()
const emit = defineEmits<{
  'update:show': [value: boolean]
  created: []
}>()

const playlists = usePlaylistStore()

const name = ref('')
const description = ref('')
const isSubmitting = ref(false)
const error = ref<string | null>(null)

function close() {
  if (isSubmitting.value) return
  name.value = ''
  description.value = ''
  error.value = null
  emit('update:show', false)
}

async function handleSubmit() {
  if (isSubmitting.value) return
  if (!name.value.trim()) {
    error.value = 'Name is required'
    return
  }
  isSubmitting.value = true
  error.value = null
  try {
    await playlists.createPlaylist(name.value.trim(), description.value.trim() || undefined)
    isSubmitting.value = false
    emit('created')
    close()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to create playlist'
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <NModal :show="show" @update:show="(show) => { if (!show) close() }" :mask-closable="!isSubmitting" :close-on-esc="!isSubmitting">
    <NCard
      style="width: 420px; max-width: 95vw"
      :title="tr('New Playlist')"
      :bordered="false"
      role="dialog"
      aria-modal="true"
    >
      <NForm @submit.prevent="handleSubmit">
        <NFormItem
          :label="tr('Name')"
          :feedback="error ? tr(error) : undefined"
          :validation-status="error ? 'error' : undefined"
        >
          <NInput
            v-model:value="name"
            :placeholder="tr('Playlist name')"
            :autofocus="true"
            @keydown.enter="handleSubmit"
          />
        </NFormItem>
        <NFormItem :label="tr('Description (optional)')">
          <NInput
            v-model:value="description"
            type="textarea"
            :placeholder="tr('Optional description')"
            :autosize="{ minRows: 2, maxRows: 4 }"
          />
        </NFormItem>
      </NForm>

      <template #footer>
        <NSpace justify="end">
          <NButton :disabled="isSubmitting" @click="close">{{ tr('Cancel') }}</NButton>
          <NButton type="primary" :loading="isSubmitting" @click="handleSubmit">{{ tr('Create') }}</NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>
</template>
