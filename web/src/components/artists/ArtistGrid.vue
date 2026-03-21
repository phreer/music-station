<script setup lang="ts">
import { NGrid, NGridItem } from 'naive-ui'
import type { Artist } from '@/types'
import ArtistCard from './ArtistCard.vue'

defineProps<{ artists: Artist[] }>()
</script>

<template>
  <NGrid :x-gap="16" :y-gap="16" cols="1 500:2 800:3 1100:4">
    <NGridItem v-for="artist in artists" :key="artist.name">
      <ArtistCard :artist="artist" />
    </NGridItem>
  </NGrid>
</template>
