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
    <NButton quaternary circle @click="ui.toggleTheme">
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
  padding: 12px 24px;
  border-bottom: 1px solid var(--app-border);
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
  font-size: 20px;
  font-weight: 700;
  background: linear-gradient(135deg, #0066cc, #00a8e8);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
</style>
