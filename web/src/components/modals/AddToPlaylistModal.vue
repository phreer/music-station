<script setup lang="ts">
import { tr } from '@/i18n'
import { ref, watch } from 'vue'
import { NModal, NCard, NButton, NSpace, NSpin, NEmpty, NAlert, useMessage } from 'naive-ui'
import { Plus } from 'lucide-vue-next'
import { usePlaylistStore } from '@/stores/playlists'
import { useAddToPlaylistStore } from '@/stores/addToPlaylist'
import CreatePlaylistModal from './CreatePlaylistModal.vue'

const playlists = usePlaylistStore()
const dialog = useAddToPlaylistStore()
const message = useMessage()
const showCreate = ref(false)
watch(() => dialog.show, () => { showCreate.value = false })

async function add(playlistId: string) {
  if (await dialog.add(playlistId)) message.success(tr('Added to playlist'))
}
</script>

<template>
  <NModal
    :show="dialog.show && !showCreate"
    :mask-closable="!dialog.submittingId"
    :close-on-esc="!dialog.submittingId"
    @update:show="(show) => { if (!show) dialog.close() }"
  >
    <NCard style="width: 420px; max-width: 95vw" :title="tr('Add to playlist')" :bordered="false" role="dialog" aria-modal="true" :aria-label="tr('Add to playlist')" :aria-busy="!!dialog.submittingId || playlists.isLoading">
      <div :class="$style.target">
        <div>{{ dialog.track?.title || tr('Unknown Title') }}</div>
        <small>{{ dialog.track?.artist || tr('Unknown Artist') }}</small>
      </div>
      <NAlert v-if="playlists.loadError" type="error" role="alert" :title="tr('Failed to load playlists')">
        {{ tr(playlists.loadError) }}
        <NButton size="small" :disabled="playlists.isLoading" @click="playlists.loadPlaylists()">{{ tr('Retry') }}</NButton>
      </NAlert>
      <NAlert v-if="dialog.error" type="error" role="alert" :title="tr('Failed to add to playlist')" style="margin-bottom: 12px">
        {{ tr(dialog.error) }}
      </NAlert>
      <NSpin :show="playlists.isLoading">
        <div v-if="playlists.isLoading" role="status" style="padding: 24px">{{ tr('Loading playlists') }}</div>
        <template v-else-if="!playlists.loadError">
          <NEmpty v-if="playlists.playlists.length === 0" :description="tr('No playlists yet')" style="padding: 24px 0" />
          <div v-else :class="$style.list">
            <button
              v-for="pl in playlists.playlists"
              :key="pl.id"
              type="button"
              :class="$style.item"
              :title="pl.name"
              :aria-label="pl.name"
              :disabled="!!dialog.submittingId || !dialog.track || pl.tracks.includes(dialog.track.id)"
              @click="add(pl.id)"
            >
              <span :class="$style.playlistName">{{ pl.name }}</span>
              <span :class="$style.playlistCount">
                <NSpin v-if="dialog.submittingId === pl.id" :size="16" />
                <template v-else-if="dialog.track && pl.tracks.includes(dialog.track.id)">{{ tr('Already added') }}</template>
                <template v-else>{{ tr('{count} tracks', { count: pl.tracks.length }) }}</template>
              </span>
            </button>
          </div>
        </template>
      </NSpin>
      <template #footer>
        <NSpace justify="space-between" align="center">
          <NButton size="small" quaternary :disabled="!!dialog.submittingId || playlists.isLoading" @click="showCreate = true">
            <template #icon><Plus :size="14" /></template>{{ tr('New Playlist') }}
          </NButton>
          <NButton :disabled="!!dialog.submittingId" @click="dialog.close">{{ tr('Cancel') }}</NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>
  <CreatePlaylistModal v-model:show="showCreate" @created="playlists.loadError && playlists.loadPlaylists()" />
</template>

<style module>
.target { margin-bottom: 16px; overflow-wrap: anywhere; }
.target small { color: var(--app-text-muted); }
.list { display: flex; flex-direction: column; gap: 4px; max-height: 320px; overflow-y: auto; padding: 3px; }
.item { display: flex; align-items: center; gap: 10px; width: 100%; padding: 10px; border: 0; border-radius: 6px; background: transparent; color: inherit; text-align: left; cursor: pointer; }
.item:hover:enabled { background: var(--app-hover); }
.item:disabled { opacity: 0.55; cursor: default; }
.item:focus-visible { outline: 2px solid var(--app-focus-ring); }
.playlistName { flex: 1; min-width: 0; font-size: 14px; overflow-wrap: anywhere; }
.playlistCount { font-size: 12px; color: var(--app-text-muted); flex-shrink: 0; }
</style>
