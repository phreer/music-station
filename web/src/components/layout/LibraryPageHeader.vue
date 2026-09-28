<script setup lang="ts">
defineProps<{ title: string; description: string }>()
</script>

<template>
  <div :class="$style.header">
    <div :class="$style.heading">
      <h1>{{ title }}</h1>
      <p>{{ description }}</p>
    </div>
    <div :class="$style.actions"><slot /></div>
  </div>
</template>

<style module>
.header { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 24px; margin-bottom: 28px; }
.heading { min-width: 0; }
.heading h1 { font-size: 30px; line-height: 1.25; font-weight: 650; letter-spacing: -1px; }
.heading p { margin-top: 7px; font-size: 13px; color: var(--app-text-muted); }
.actions { display: flex; align-items: center; gap: 10px; min-width: 0; max-width: 100%; margin-left: auto; }
@media (max-width: 800px) {
  .header { flex-wrap: wrap; gap: 18px; margin-bottom: 24px; }
  .actions { flex: 1 1 100%; }
  .heading h1 { font-size: 27px; }
}
</style>
