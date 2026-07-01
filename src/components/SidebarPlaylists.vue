<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { open } from '@tauri-apps/plugin-dialog'
import { Library, Music, Plus, Upload, Heart, MoreHorizontal } from 'lucide-vue-next'
import { useLibraryStore } from '../stores/library'
import { usePlayerStore } from '../stores/player'
import { useContextMenu } from '../composables/useContextMenu'
import { buildPlaylistMenuItems } from '../composables/usePlaylistContextMenu'
import SidebarItem from './SidebarItem.vue'
import CreatePlaylistDialog from './CreatePlaylistDialog.vue'
import ConfirmDialog from './ConfirmDialog.vue'
import ContextMenu from './ContextMenu.vue'
import * as api from '../lib/invoke'
import type { Playlist } from '../lib/invoke'

const router = useRouter()
const library = useLibraryStore()
const playerStore = usePlayerStore()
const contextMenu = useContextMenu()

const createDialogOpen = ref(false)
const renameDialogOpen = ref(false)
const deleteConfirmOpen = ref(false)
const importDialogOpen = ref(false)
const playlistToDelete = ref<Playlist | null>(null)
const renamingId = ref('')
const renamingName = ref('')
const importFilePath = ref('')
const importPlaylistName = ref('')
const isImporting = ref(false)

async function handleCreate(name: string) {
  const p = await library.createPlaylist(name)
  if (p) router.push(`/playlists/${p.id}`)
}

async function handleImportClick() {
  try {
    const selected = await open({
      filters: [{ name: 'M3U Playlist', extensions: ['m3u', 'm3u8'] }],
      multiple: false,
    })
    if (!selected) return
    const filePath = selected as string
    const baseName = filePath.split(/[/\\]/).pop()?.replace(/\.(m3u8?|M3U8?)$/, '') || 'Imported Playlist'
    importFilePath.value = filePath
    importPlaylistName.value = baseName
    importDialogOpen.value = true
  } catch (e) {
    console.error('Failed to select M3U file', e)
  }
}

async function handleImportConfirm(name: string) {
  if (!importFilePath.value || isImporting.value) return
  isImporting.value = true
  try {
    const result = await library.importPlaylist(importFilePath.value, name)
    if (result) router.push(`/playlists/${result.id}`)
  } finally {
    isImporting.value = false
    importFilePath.value = ''
    importPlaylistName.value = ''
  }
}

function openRenameDialog(id: string, name: string) {
  renamingId.value = id
  renamingName.value = name
  renameDialogOpen.value = true
}

async function handleRename(name: string) {
  if (renamingId.value) await library.renamePlaylist(renamingId.value, name)
}

function openDeleteConfirm(playlist: Playlist) {
  playlistToDelete.value = playlist
  deleteConfirmOpen.value = true
}

async function handleDelete() {
  if (playlistToDelete.value) {
    await library.deletePlaylist(playlistToDelete.value.id)
    if (router.currentRoute.value.params.id === playlistToDelete.value.id) {
      router.push('/')
    }
    playlistToDelete.value = null
  }
}

function playPlaylist(playlist: Playlist) {
  api.getPlaylistTracks(playlist.id).then(tracks => {
    if (tracks.length > 0) playerStore.playTracks(tracks, 0)
  })
}

async function exportPlaylist(playlist: Playlist) {
  try {
    const { save } = await import('@tauri-apps/plugin-dialog')
    const path = await save({
      filters: [{ name: 'M3U Playlist', extensions: ['m3u'] }],
      defaultPath: `${playlist.name}.m3u`,
    })
    if (path) await api.exportPlaylistM3U(playlist.id, path)
  } catch (e) {
    console.error('Export failed', e)
  }
}

function openPlaylistContextMenu(playlist: Playlist, e: MouseEvent) {
  contextMenu.open(e, buildPlaylistMenuItems(playlist, {
    onPlay: playPlaylist,
    onRename: (p) => openRenameDialog(p.id, p.name),
    onDelete: openDeleteConfirm,
    onExport: exportPlaylist,
  }))
}
</script>

<template>
  <div class="flex-1 overflow-y-auto px-3 pb-2">
    <div class="sticky top-0 z-10 flex items-center justify-between px-3 py-2">
      <div class="flex items-center gap-2 text-foreground opacity-80">
        <Library class="w-3.5 h-3.5" />
        <span class="text-xs font-semibold uppercase tracking-widest">Playlists</span>
      </div>
      <div class="flex items-center gap-1">
        <button
          class="w-6 h-6 flex items-center justify-center rounded text-foreground opacity-80 hover:text-foreground hover:bg-foreground/[0.06] transition-colors"
          title="Import M3U playlist"
          @click.stop="handleImportClick">
          <Upload class="w-3.5 h-3.5" />
        </button>
        <button
          class="w-6 h-6 flex items-center justify-center rounded text-foreground opacity-80 hover:text-foreground hover:bg-foreground/[0.06] transition-colors"
          title="New playlist"
          @click.stop="createDialogOpen = true">
          <Plus class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <div class="space-y-0.5">
      <SidebarItem to="/playlists/favorites" :icon="Heart" label="Favorites" />

      <SidebarItem
        v-for="playlist in library.playlists"
        :key="playlist.id"
        :to="`/playlists/${playlist.id}`"
        :icon="Music"
        :label="playlist.name"
        @contextmenu="openPlaylistContextMenu(playlist, $event)">
        <template #actions>
          <button
            class="w-6 h-6 flex items-center justify-center rounded text-foreground opacity-0 group-hover:opacity-100 hover:!text-foreground hover:bg-foreground/[0.08] transition-colors"
            @click.stop="(e) => openPlaylistContextMenu(playlist, e)">
            <MoreHorizontal class="w-3.5 h-3.5" />
          </button>
        </template>
      </SidebarItem>

      <p v-if="library.playlists.length === 0" class="px-3 py-2 text-xs text-foreground opacity-50">
        No playlists yet
      </p>
    </div>
  </div>

  <CreatePlaylistDialog v-model:open="createDialogOpen" @confirm="handleCreate" />
  <CreatePlaylistDialog
    v-model:open="renameDialogOpen"
    :initial-name="renamingName"
    title="Rename Playlist"
    confirm-label="Save"
    @confirm="handleRename" />
  <CreatePlaylistDialog
    v-model:open="importDialogOpen"
    :initial-name="importPlaylistName"
    title="Import Playlist"
    confirm-label="Import"
    @confirm="handleImportConfirm" />
  <ConfirmDialog
    v-model:open="deleteConfirmOpen"
    title="Delete Playlist"
    message="Delete this playlist? Tracks won't be removed from your library."
    confirm-label="Delete"
    :danger="true"
    @confirm="handleDelete" />
  <ContextMenu
    :visible="contextMenu.visible.value"
    :x="contextMenu.x.value"
    :y="contextMenu.y.value"
    :items="contextMenu.items.value"
    @close="contextMenu.close()" />
</template>
