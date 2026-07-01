<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ListMusic, Play, Plus, Trash2, Download, Heart } from 'lucide-vue-next'
import { useLibraryStore } from '../stores/library'
import { usePlayerStore } from '../stores/player'
import { useContextMenu } from '../composables/useContextMenu'
import ContextMenu from '../components/ContextMenu.vue'
import ConfirmDialog from '../components/ConfirmDialog.vue'
import * as api from '../lib/invoke'

const router = useRouter()
const library = useLibraryStore()
const playerStore = usePlayerStore()
const menu = useContextMenu()
const showNewPlaylist = ref(false)
const newName = ref('')
const showDeleteConfirm = ref(false)
const pendingDeleteId = ref('')
const pendingDeleteName = ref('')

const trackCounts = ref<Record<string, number>>({})

onMounted(async () => {
  if (library.playlists.length === 0) await library.loadPlaylists()
  for (const p of library.playlists) {
    try {
      trackCounts.value[p.id] = await api.getPlaylistTrackCount(p.id)
    } catch {
      trackCounts.value[p.id] = 0
    }
  }
})

function navigate(id: string) {
  router.push({ name: 'playlist-detail', params: { id } })
}

function navigateFavorites() {
  router.push('/playlists/favorites')
}

function playFavorites() {
  api.getFavoriteIds().then(ids => {
    const idSet = new Set(ids)
    const favTracks = library.tracks.filter(t => idSet.has(t.id))
    if (favTracks.length > 0) playerStore.playTracks(favTracks, 0)
  })
}

function playPlaylist(id: string) {
  api.getPlaylistTracks(id).then(tracks => {
    if (tracks.length > 0) playerStore.playTracks(tracks, 0)
  })
}

async function createPlaylist() {
  if (!newName.value.trim()) return
  await api.createPlaylist(newName.value.trim())
  newName.value = ''
  showNewPlaylist.value = false
  await library.loadPlaylists()
}

function promptDelete(id: string, name: string) {
  pendingDeleteId.value = id
  pendingDeleteName.value = name
  showDeleteConfirm.value = true
}

async function doDelete() {
  if (pendingDeleteId.value) {
    await api.deletePlaylist(pendingDeleteId.value)
    await library.loadPlaylists()
  }
}

async function promptExport(id: string) {
  try {
    const { save } = await import('@tauri-apps/plugin-dialog')
    const path = await save({
      filters: [{ name: 'M3U Playlist', extensions: ['m3u'] }],
      defaultPath: `${library.playlists.find(p => p.id === id)?.name || 'playlist'}.m3u`,
    })
    if (path) {
      await api.exportPlaylistM3U(id, path)
    }
  } catch (e) {
    console.error('Export failed', e)
  }
}

function onContextMenu(e: MouseEvent, playlist: { id: string; name: string }) {
  menu.open(e, [
    { label: 'Play', icon: Play, action: () => playPlaylist(playlist.id) },
    { separator: true },
    {
      label: 'Export as M3U',
      icon: Download,
      action: () => promptExport(playlist.id),
    },
    { separator: true },
    {
      label: 'Delete',
      icon: Trash2,
      danger: true,
      action: () => promptDelete(playlist.id, playlist.name),
    },
  ])
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden">
    <div class="flex items-center justify-between px-6 py-4 border-b border-foreground/[0.06]">
      <h1 class="text-xl font-bold">Playlists</h1>
      <button @click="showNewPlaylist = true"
        class="flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:opacity-90 transition-opacity">
        <Plus class="w-4 h-4" />
        New Playlist
      </button>
    </div>

    <div class="flex-1 overflow-y-auto custom-scrollbar p-6">
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
        <div class="group cursor-pointer" @click="navigateFavorites">
          <div class="aspect-square bg-rose-500/10 rounded-lg ring-1 ring-rose-500/20 overflow-hidden relative flex items-center justify-center mb-3 transition-all group-hover:ring-rose-500/40">
            <Heart class="w-1/3 h-1/3 text-rose-500 opacity-80 group-hover:opacity-100 transition-opacity" />
            <div class="absolute inset-0 bg-background/20 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
              <button @click.stop="playFavorites"
                class="w-12 h-12 bg-foreground text-background rounded-full shadow-xl flex items-center justify-center transform translate-y-4 group-hover:translate-y-0 transition-all duration-300">
                <Play class="w-6 h-6 fill-current ml-1" />
              </button>
            </div>
          </div>
          <div class="px-1">
            <h3 class="font-medium text-sm truncate">Favorites</h3>
            <p class="text-xs text-foreground opacity-50 truncate mt-0.5">Liked tracks</p>
          </div>
        </div>

        <div v-for="p in library.playlists" :key="p.id"
          class="group cursor-pointer" @click="navigate(p.id)"
          @contextmenu.prevent="onContextMenu($event, p)">
          <div class="aspect-square bg-foreground/5 rounded-lg ring-1 ring-foreground/[0.06] overflow-hidden relative flex items-center justify-center mb-3 transition-all group-hover:ring-primary/30">
            <ListMusic class="w-1/3 h-1/3 text-foreground opacity-40 group-hover:opacity-60 transition-opacity" />
            <div class="absolute inset-0 bg-background/20 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
              <button @click.stop="playPlaylist(p.id)"
                class="w-12 h-12 bg-foreground text-background rounded-full shadow-xl flex items-center justify-center transform translate-y-4 group-hover:translate-y-0 transition-all duration-300">
                <Play class="w-6 h-6 fill-current ml-1" />
              </button>
            </div>
          </div>
          <div class="px-1">
            <h3 class="font-medium text-sm truncate">{{ p.name }}</h3>
            <p class="text-xs text-foreground opacity-50 truncate mt-0.5">{{ trackCounts[p.id] || 0 }} tracks</p>
          </div>
        </div>
      </div>
    </div>

    <!-- New Playlist Dialog -->
    <div v-if="showNewPlaylist" class="fixed inset-0 z-50 flex items-center justify-center bg-background/60 backdrop-blur-sm"
      @click.self="showNewPlaylist = false">
      <div class="bg-background border border-foreground/[0.1] rounded-xl p-6 shadow-2xl w-80" @click.stop>
        <h2 class="text-lg font-bold mb-4">New Playlist</h2>
        <input v-model="newName" placeholder="Playlist name" autofocus
          class="w-full px-3 py-2 rounded-lg bg-foreground/5 border border-foreground/[0.1] text-sm outline-none focus:border-primary/50 transition-colors"
          @keydown.enter="createPlaylist" />
        <div class="flex justify-end gap-2 mt-4">
          <button @click="showNewPlaylist = false"
            class="px-3 py-1.5 text-sm rounded-lg hover:bg-foreground/10 transition-colors">Cancel</button>
          <button @click="createPlaylist" :disabled="!newName.trim()"
            class="px-3 py-1.5 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:opacity-90 transition-opacity disabled:opacity-40">Create</button>
        </div>
      </div>
    </div>

    <ContextMenu :visible="menu.visible.value" :x="menu.x.value" :y="menu.y.value"
      :items="menu.items.value" @close="menu.close()" />

    <ConfirmDialog :open="showDeleteConfirm" title="Delete Playlist"
      :message="`Delete &quot;${pendingDeleteName}&quot;? Tracks won't be removed from your library.`"
      confirmLabel="Delete" :danger="true"
      @update:open="showDeleteConfirm = $event"
      @confirm="doDelete()" @cancel="showDeleteConfirm = false" />
  </div>
</template>
