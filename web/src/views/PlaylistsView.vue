<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { NButton, NEmpty, NGrid, NGridItem, NSpin } from 'naive-ui'
import { Plus } from 'lucide-vue-next'
import { usePlaylistStore } from '@/stores/playlists'
import PlaylistCard from '@/components/playlists/PlaylistCard.vue'
import CreatePlaylistModal from '@/components/modals/CreatePlaylistModal.vue'

const playlistStore = usePlaylistStore()
const showCreate = ref(false)

onMounted(() => {
  if (playlistStore.playlists.length === 0) {
    playlistStore.loadPlaylists()
  }
})
</script>

<template>
  <div :class="$style.container">
    <div :class="$style.toolbar">
      <h2 :class="$style.heading">Playlists</h2>
      <NButton type="primary" size="small" @click="showCreate = true">
        <template #icon><Plus :size="16" /></template>
        New Playlist
      </NButton>
    </div>

    <NSpin :show="playlistStore.isLoading">
      <NEmpty
        v-if="!playlistStore.isLoading && playlistStore.playlists.length === 0"
        description="No playlists yet"
        style="padding: 60px 0"
      />
      <NGrid v-else :x-gap="16" :y-gap="16" cols="1 600:2 900:3 1200:4">
        <NGridItem v-for="playlist in playlistStore.playlists" :key="playlist.id">
          <PlaylistCard :playlist="playlist" />
        </NGridItem>
      </NGrid>
    </NSpin>

    <CreatePlaylistModal v-model:show="showCreate" />
  </div>
</template>

<style module>
.container {
  padding: 24px;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.heading {
  font-size: 22px;
  font-weight: 700;
}
</style>
