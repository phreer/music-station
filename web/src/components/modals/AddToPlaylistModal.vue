<script setup lang="ts">
import { tr } from '@/i18n'
import { ref, watch } from 'vue'
import { NModal, NCard, NButton, NSpace, NSpin, NEmpty, NAlert, useMessage } from 'naive-ui'
import { Check, ListMusic, Music2, Plus } from 'lucide-vue-next'
import { coverUrl } from '@/api/client'
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
        <div :class="$style.trackCover">
          <img v-if="dialog.track?.has_cover" :src="coverUrl(dialog.track.id)" alt="" />
          <Music2 v-else :size="15" aria-hidden="true" />
        </div>
        <span :class="$style.trackTitle" :title="dialog.track?.title || tr('Unknown Title')">{{ dialog.track?.title || tr('Unknown Title') }}</span>
        <span :class="$style.trackSeparator" aria-hidden="true">·</span>
        <span :class="$style.trackArtist" :title="dialog.track?.artist || tr('Unknown Artist')">{{ dialog.track?.artist || tr('Unknown Artist') }}</span>
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
              <span :class="$style.playlistIcon"><ListMusic :size="20" aria-hidden="true" /></span>
              <span :class="$style.playlistInfo">
                <span :class="$style.playlistName">{{ pl.name }}</span>
                <span :class="$style.playlistCount">{{ tr('{count} tracks', { count: pl.tracks.length }) }}</span>
              </span>
              <span :class="$style.playlistAction">
                <NSpin v-if="dialog.submittingId === pl.id" :size="16" />
                <template v-else-if="dialog.track && pl.tracks.includes(dialog.track.id)"><Check :size="16" aria-hidden="true" />{{ tr('Already added') }}</template>
                <Plus v-else :size="18" aria-hidden="true" />
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
.target { display: flex; align-items: center; gap: 8px; min-width: 0; margin-bottom: 16px; font-size: 12px; }
.trackCover { display: grid; place-items: center; flex-shrink: 0; width: 28px; height: 28px; overflow: hidden; border-radius: 6px; background: var(--app-placeholder-bg); color: var(--app-text-muted); }
.trackCover img { width: 100%; height: 100%; object-fit: cover; }
.trackTitle, .trackArtist { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.trackTitle { font-weight: 500; }
.trackArtist { flex: 0 1 40%; color: var(--app-text-muted); }
.trackSeparator { flex-shrink: 0; color: var(--app-text-muted); }
.list { display: flex; flex-direction: column; gap: 8px; max-height: 320px; overflow-y: auto; padding: 3px; }
.item { display: flex; align-items: center; gap: 12px; width: 100%; min-height: 64px; padding: 12px; border: 1px solid var(--app-border); border-radius: 10px; background: var(--app-surface); color: inherit; text-align: left; font: inherit; cursor: pointer; transition: border-color 0.15s, background 0.15s; }
.item:hover:enabled { border-color: var(--app-primary); background: var(--app-active-bg); }
.item:disabled { color: var(--app-text-muted); cursor: default; }
.item:disabled .playlistIcon { background: var(--app-placeholder-bg); color: var(--app-text-muted); }
.item:focus-visible { outline: 2px solid var(--app-focus-ring); outline-offset: 1px; }
.playlistIcon { display: grid; place-items: center; flex-shrink: 0; width: 36px; height: 36px; border-radius: 8px; background: var(--app-active-bg); color: var(--app-primary); }
.playlistInfo { display: flex; flex-direction: column; flex: 1; min-width: 0; gap: 4px; }
.playlistName { font-size: 14px; font-weight: 600; overflow-wrap: anywhere; }
.playlistCount { font-size: 12px; color: var(--app-text-muted); }
.playlistAction { display: flex; align-items: center; gap: 4px; flex-shrink: 0; font-size: 12px; color: var(--app-primary); }
.item:disabled .playlistAction { color: var(--app-text-muted); }
</style>
