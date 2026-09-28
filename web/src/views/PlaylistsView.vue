<script setup lang="ts">
import LibraryPageHeader from '@/components/layout/LibraryPageHeader.vue'
import { ref, onMounted } from 'vue'
import { NButton, NEmpty, NSpin, NModal } from 'naive-ui'
import { Plus } from 'lucide-vue-next'
import { usePlaylistStore } from '@/stores/playlists'
import PlaylistCard from '@/components/playlists/PlaylistCard.vue'
import CreatePlaylistModal from '@/components/modals/CreatePlaylistModal.vue'

const playlistStore = usePlaylistStore()
const showCreate = ref(false)

// Shared delete confirmation state (lifted from per-card NPopconfirm)
const deleteTarget = ref<{ id: string; name: string } | null>(null)

function requestDelete(playlistId: string, playlistName: string) {
  deleteTarget.value = { id: playlistId, name: playlistName }
}

async function confirmDelete() {
  if (!deleteTarget.value) return
  await playlistStore.deletePlaylist(deleteTarget.value.id)
  deleteTarget.value = null
}

function cancelDelete() {
  deleteTarget.value = null
}

onMounted(() => {
  if (playlistStore.playlists.length === 0) {
    playlistStore.loadPlaylists()
  }
})
</script>

<template>
  <div class="library-page">
    <LibraryPageHeader title="Playlists" :description="`${playlistStore.playlists.length} collections, made by you`">
      <NButton type="primary" @click="showCreate = true">
        <template #icon><Plus :size="16" /></template>
        New Playlist
      </NButton>
    </LibraryPageHeader>

    <NSpin :show="playlistStore.isLoading">
      <NEmpty
        v-if="!playlistStore.isLoading && playlistStore.playlists.length === 0"
        description="No playlists yet"
        style="padding: 60px 0"
      />
      <div v-else :class="$style.grid">
        <PlaylistCard v-for="playlist in playlistStore.playlists" :key="playlist.id" :playlist="playlist" @request-delete="requestDelete" />
      </div>
    </NSpin>

    <CreatePlaylistModal v-model:show="showCreate" />

    <!-- Shared delete confirmation dialog -->
    <NModal
      :show="deleteTarget !== null"
      preset="dialog"
      type="warning"
      title="Delete Playlist"
      :content="`Delete playlist &quot;${deleteTarget?.name ?? ''}&quot;? This cannot be undone.`"
      positive-text="Delete"
      negative-text="Cancel"
      @positive-click="confirmDelete"
      @negative-click="cancelDelete"
      @mask-click="cancelDelete"
      @close="cancelDelete"
    />
  </div>
</template>

<style module>
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(100%, 220px), 1fr)); gap: 28px 24px; align-items: start; }
@media (max-width: 600px) { .grid { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 24px 16px; } }
</style>
