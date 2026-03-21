<script setup lang="ts">
import { ref, computed } from 'vue'
import { NModal, NCard, NButton, NSpace, NSpin, NEmpty, NCheckbox } from 'naive-ui'
import { Plus } from 'lucide-vue-next'
import type { Track } from '@/types'
import { usePlaylistStore } from '@/stores/playlists'
import CreatePlaylistModal from './CreatePlaylistModal.vue'

const props = defineProps<{
  show: boolean
  track: Track | null
}>()
const emit = defineEmits<{
  'update:show': [value: boolean]
}>()

const playlists = usePlaylistStore()
const showCreate = ref(false)
const addingIds = ref<Set<string>>(new Set())

const alreadyInPlaylist = computed(() => {
  if (!props.track) return new Set<string>()
  const s = new Set<string>()
  for (const pl of playlists.playlists) {
    if (pl.tracks.includes(props.track.id)) s.add(pl.id)
  }
  return s
})

function close() {
  emit('update:show', false)
}

async function togglePlaylist(playlistId: string) {
  if (!props.track) return
  addingIds.value.add(playlistId)
  try {
    if (alreadyInPlaylist.value.has(playlistId)) {
      await playlists.removeTrack(playlistId, props.track.id)
    } else {
      await playlists.addTrack(playlistId, props.track.id)
    }
  } finally {
    addingIds.value.delete(playlistId)
  }
}
</script>

<template>
  <NModal :show="show" @update:show="emit('update:show', $event)" :mask-closable="true">
    <NCard
      style="width: 380px; max-width: 95vw"
      title="Add to Playlist"
      :bordered="false"
      role="dialog"
      aria-modal="true"
    >
      <NSpin :show="playlists.isLoading">
        <NEmpty
          v-if="!playlists.isLoading && playlists.playlists.length === 0"
          description="No playlists yet"
          style="padding: 24px 0"
        />
        <div v-else :class="$style.list">
          <div
            v-for="pl in playlists.playlists"
            :key="pl.id"
            :class="$style.item"
            @click="togglePlaylist(pl.id)"
          >
            <NCheckbox
              :checked="alreadyInPlaylist.has(pl.id)"
              :disabled="addingIds.has(pl.id)"
              @update:checked="togglePlaylist(pl.id)"
              @click.stop
            />
            <span :class="$style.playlistName">{{ pl.name }}</span>
            <span :class="$style.playlistCount">{{ pl.tracks.length }} tracks</span>
          </div>
        </div>
      </NSpin>

      <template #footer>
        <NSpace justify="space-between" align="center">
          <NButton size="small" quaternary @click="showCreate = true">
            <template #icon><Plus :size="14" /></template>
            New Playlist
          </NButton>
          <NButton @click="close">Done</NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>

  <CreatePlaylistModal v-model:show="showCreate" />
</template>

<style module>
.list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-height: 320px;
  overflow-y: auto;
}

.item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.item:hover {
  background: var(--app-hover);
}

.playlistName {
  flex: 1;
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.playlistCount {
  font-size: 12px;
  opacity: 0.5;
  flex-shrink: 0;
}
</style>
