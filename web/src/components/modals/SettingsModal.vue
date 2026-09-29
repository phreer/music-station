<script setup lang="ts">
import { NModal, NCard, NForm, NFormItem, NSelect, NSwitch, NButton } from 'naive-ui'
import { language, setLanguage, tr } from '@/i18n'
import { useUiStore } from '@/stores/ui'

defineProps<{ show: boolean }>()
const emit = defineEmits<{ 'update:show': [value: boolean] }>()
const ui = useUiStore()
const languageOptions = [
  { label: 'English', value: 'en' },
  { label: '简体中文', value: 'zh' },
]
</script>

<template>
  <NModal :show="show" @update:show="emit('update:show', $event)">
    <NCard
      style="width: 420px; max-width: 94vw"
      :title="tr('Settings')"
      :bordered="false"
      role="dialog"
      aria-modal="true"
      :aria-label="tr('Settings')"
      closable
      @close="emit('update:show', false)"
    >
      <NForm label-placement="top">
        <NFormItem :label="tr('Language')">
          <NSelect
            :value="language"
            :options="languageOptions"
            :aria-label="tr('Language')"
            @update:value="setLanguage"
          />
        </NFormItem>
        <NFormItem :label="tr('Appearance')">
          <NSwitch :value="ui.isDarkMode" :aria-label="tr('Dark theme')" @update:value="ui.toggleTheme" />
          <span style="margin-left: 12px">{{ tr('Dark theme') }}</span>
        </NFormItem>
      </NForm>
      <template #footer>
        <NButton @click="emit('update:show', false)">{{ tr('Close') }}</NButton>
      </template>
    </NCard>
  </NModal>
</template>
