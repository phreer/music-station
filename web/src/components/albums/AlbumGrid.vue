<script setup lang="ts">
import { NGrid, NGridItem } from 'naive-ui'
import type { Album } from '@/types'
import AlbumCard from './AlbumCard.vue'

defineProps<{ albums: Album[] }>()
</script>

<template>
  <NGrid :x-gap="16" :y-gap="16" cols="1 500:2 800:3 1100:4">
    <NGridItem v-for="album in albums" :key="album.name + album.artist">
      <AlbumCard :album="album" />
    </NGridItem>
  </NGrid>
</template>
