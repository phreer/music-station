<script setup lang="ts">
import { Music } from 'lucide-vue-next'
import { NButton } from 'naive-ui'
import { Sun, Moon } from 'lucide-vue-next'
import { useUiStore } from '@/stores/ui'

const ui = useUiStore()
</script>

<template>
  <header :class="$style.header">
    <div :class="$style.titleGroup">
      <Music :size="28" :class="$style.logo" />
      <h1 :class="$style.title">Music Station</h1>
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
  border-bottom: 1px solid var(--n-border-color, #e0e0e0);
}

.titleGroup {
  display: flex;
  align-items: center;
  gap: 10px;
}

.logo {
  color: var(--n-primary-color, #0066cc);
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
