<script setup lang="ts">
import { NButton } from 'naive-ui'
import { Sun, Moon } from 'lucide-vue-next'
import { useUiStore } from '@/stores/ui'

const ui = useUiStore()
</script>

<template>
  <header :class="$style.header">
    <div :class="$style.titleGroup">
      <img src="/favicon.png" :class="$style.logo" alt="FL Music" />
      <h1 :class="$style.title">FL Music</h1>
    </div>
    <NButton quaternary circle :aria-label="ui.isDarkMode ? 'Switch to light theme' : 'Switch to dark theme'" @click="ui.toggleTheme">
      <template #icon>
        <Moon v-if="ui.isDarkMode" :size="18" />
        <Sun v-else :size="18" />
      </template>
    </NButton>
  </header>
</template>

<style module>
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 56px;
  padding: 8px 24px;
  background: var(--app-surface);
}

.titleGroup {
  display: flex;
  align-items: center;
  gap: 10px;
}

.logo {
  width: 28px;
  height: 28px;
  object-fit: contain;
}

.title {
  font-size: 19px;
  font-weight: 700;
  letter-spacing: -0.025em;
  color: var(--app-text);
}

@media (max-width: 600px) {
  .header { padding-inline: 16px; }
}
</style>
