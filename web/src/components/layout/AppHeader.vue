<script setup lang="ts">
import { tr } from '@/i18n'
import { ref } from 'vue'
import SettingsModal from '@/components/modals/SettingsModal.vue'
import { NButton } from 'naive-ui'
import { Sun, Moon, AudioLines, Settings } from 'lucide-vue-next'
import { useUiStore } from '@/stores/ui'
import AppNav from './AppNav.vue'

const ui = useUiStore()
const showSettings = ref(false)
</script>

<template>
  <header :class="$style.header">
    <RouterLink :to="{ name: 'tracks' }" :class="$style.brand" :aria-label="tr('FL Music library')">
      <span :class="$style.mark"><AudioLines :size="21" :stroke-width="1.8" /></span>
      <span>FL Music<span :class="$style.brandDot">.</span></span>
    </RouterLink>
    <div :class="$style.navigation"><AppNav /></div>
    <div :class="$style.headerActions">
      <NButton quaternary circle :aria-label="ui.isDarkMode ? tr('Switch to light theme') : tr('Switch to dark theme')" @click="ui.toggleTheme">
        <template #icon>
          <Sun v-if="ui.isDarkMode" :size="18" />
          <Moon v-else :size="18" />
        </template>
      </NButton>
      <NButton quaternary circle :aria-label="tr('Settings')" :title="tr('Settings')" @click="showSettings = true">
        <template #icon><Settings :size="18" /></template>
      </NButton>
    </div>
  </header>
  <SettingsModal v-model:show="showSettings" />
</template>

<style module>
.header { display: grid; grid-template-columns: 1fr auto 1fr; align-items: center; gap: 24px; padding: 18px 36px; border-bottom: 1px solid var(--app-border); background: var(--app-surface); flex-shrink: 0; }
.brand { display: inline-flex; align-items: center; gap: 10px; width: fit-content; font-size: 19px; font-weight: 700; letter-spacing: -0.6px; color: var(--app-text); text-decoration: none; white-space: nowrap; }
.mark { display: grid; place-items: center; width: 33px; height: 33px; border-radius: 10px; color: var(--app-on-primary); background: var(--app-primary); }
.brandDot { color: var(--app-primary); }
.headerActions { display: flex; gap: 8px; justify-self: end; }
@media (max-width: 1000px) {
  .header { grid-template-columns: auto 1fr auto; gap: 16px; padding: 14px 24px; }
  .navigation { justify-self: center; }
}
@media (max-width: 800px) {
  .header { grid-template-columns: 1fr auto; padding: 12px 16px; gap: 12px; }
  .navigation { grid-row: 2; grid-column: 1 / -1; width: 100%; }
  .headerActions { grid-row: 1; grid-column: 2; }
  .brand { font-size: 18px; }
}
</style>
