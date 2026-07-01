<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useLibraryStore } from '../stores/library'
import { usePlayerStore } from '../stores/player'
import { useFavoritesStore } from '../stores/favorites'
import { Play, Shuffle, MoreVertical, Clock, Music, Search } from 'lucide-vue-next'
import DetailHero from '../components/DetailHero.vue'
import TrackTable from '../components/TrackTable.vue'
import CreatePlaylistDialog from '../components/CreatePlaylistDialog.vue'
import ConfirmDialog from '../components/ConfirmDialog.vue'
import { useContextMenu } from '../composables/useContextMenu'
import ContextMenu from '../components/ContextMenu.vue'
import * as api from '../lib/invoke'

const route = useRoute()
const router = useRouter()
const library = useLibraryStore()
const playerStore = usePlayerStore()
const favoritesStore = useFavoritesStore()

const playlistId = computed(() => route.params.id as string)
const isFavorites = computed(() => playlistId.value === 'favorites')

const playlist = computed(() => {
  if (isFavorites.value) return { id: 'favorites', name: 'Favorites', description: '', created_at: '' }
  return library.playlists.find(p => p.id === playlistId.value)
})

const tracks = ref<api.Track[]>([])
const isLoading = ref(true)
const searchQuery = ref('')

const filteredTracks = computed(() => {
  if (!searchQuery.value) return tracks.value
  const q = searchQuery.value.toLowerCase()
  return tracks.value.filter(t =>
    (t.title || '').toLowerCase().includes(q) ||
    (t.artist || '').toLowerCase().includes(q) ||
    (t.album || '').toLowerCase().includes(q)
  )
})

async function loadTracks(silent = false) {
  if (!silent) isLoading.value = true
  try {
    if (isFavorites.value) {
      const ids = await api.getFavoriteIds()
      const idSet = new Set(ids)
      tracks.value = library.tracks.filter(t => idSet.has(t.id))
    } else {
      tracks.value = await api.getPlaylistTracks(playlistId.value)
    }
  } catch {}
  if (!silent) isLoading.value = false
}

watch(playlistId, () => { loadTracks(); searchQuery.value = '' }, { immediate: true })
watch(() => favoritesStore.favoriteIds.size, () => {
  if (isFavorites.value) loadTracks(true)
})

const renameDialogOpen = ref(false)
const deleteConfirmOpen = ref(false)

async function handleRename(name: string) {
  if (!isFavorites.value && playlist.value) {
    await library.renamePlaylist(playlist.value.id, name)
  }
}

async function handleDelete() {
  if (!isFavorites.value && playlist.value) {
    await library.deletePlaylist(playlist.value.id)
    router.push('/')
  }
}

function playPlaylist() {
  if (filteredTracks.value.length > 0) playerStore.playTracks(filteredTracks.value, 0)
}

function shufflePlaylist() {
  if (filteredTracks.value.length > 0) {
    const shuffled = [...filteredTracks.value]
    for (let i = shuffled.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]]
    }
    playerStore.playTracks(shuffled, 0)
  }
}

async function handleReorder(newTracks: api.Track[]) {
  if (!playlist.value || isFavorites.value || !playlistId.value) return
  const trackIds = newTracks.map(t => t.id)
  try {
    await api.reorderPlaylistTracks(playlistId.value, trackIds)
    tracks.value = newTracks
  } catch (e) {
    loadTracks(true)
  }
}

const contextMenu = useContextMenu()

function openContextMenu(e: MouseEvent) {
  contextMenu.open(e, [
    { label: 'Play', icon: Play, action: () => playPlaylist() },
    { label: 'Shuffle', icon: Shuffle, action: () => shufflePlaylist() },
    { separator: true },
    ...(!isFavorites.value ? [
      { label: 'Rename', icon: MoreVertical, action: () => renameDialogOpen.value = true },
      { label: 'Delete', icon: MoreVertical, action: () => deleteConfirmOpen.value = true, danger: true },
    ] : []),
  ] as any)
}

const totalDuration = computed(() => {
  const total = tracks.value.reduce((a, t) => a + t.duration, 0)
  return `${Math.floor(total / 60)} min`
})
</script>

<template>
  <div class="h-full flex flex-col bg-background overflow-hidden">
    <div v-if="isLoading" class="flex-1 flex items-center justify-center">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
    </div>

    <div v-else-if="playlist" class="flex-1 overflow-y-auto custom-scrollbar">
      <DetailHero :title="playlist.name">
        <template #top-right>
          <div class="relative max-w-sm w-full">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-foreground opacity-60" />
            <input v-model="searchQuery" type="text" placeholder="Search in playlist..."
              class="w-full pl-10 pr-4 py-2 text-sm rounded-xl bg-foreground/[0.04] border border-foreground/10 outline-none focus:border-primary/50" />
          </div>
        </template>
        <template #artwork>
          <div class="w-48 h-48 rounded-lg shadow-2xl overflow-hidden ring-1 ring-foreground/[0.08] bg-foreground/5 flex-shrink-0 flex items-center justify-center">
            <Music v-if="isFavorites" class="w-16 h-16 text-rose-500 opacity-80" />
            <Music v-else class="w-16 h-16 text-foreground opacity-40" />
          </div>
        </template>
        <template #metadata>
          <div class="flex gap-2 text-sm items-end flex-wrap">
            <div class="flex items-center gap-2">
              <Music class="w-4 h-4" />
              <span>{{ tracks.length }} tracks</span>
            </div>
            <div class="flex items-center gap-2">
              <Clock class="w-4 h-4" />
              <span>{{ totalDuration }}</span>
            </div>
          </div>
        </template>
        <template #actions>
          <button @click="playPlaylist"
            class="flex items-center gap-2 px-6 py-2.5 bg-primary text-primary-foreground rounded-full font-medium hover:scale-105 transition-transform shadow-lg shadow-primary/20">
            <Play class="w-5 h-5 fill-current" /> Play
          </button>
          <div class="flex gap-2">
            <button @click="shufflePlaylist"
              class="flex items-center gap-2 px-5 py-2.5 bg-foreground/10 text-foreground rounded-full font-medium hover:bg-foreground/20 transition-all">
              <Shuffle class="w-4 h-4" />
            </button>
            <button @click="openContextMenu"
              class="flex items-center gap-2 px-3 py-2.5 bg-foreground/10 text-foreground rounded-full font-medium hover:bg-foreground/20 transition-all">
              <MoreVertical class="w-4 h-4" />
            </button>
          </div>
        </template>
      </DetailHero>

      <div class="h-[calc(100vh-390px)]">
        <TrackTable
          :tracks="filteredTracks"
          :show-artwork="true"
          :simple-mode="true"
          :allow-dnd="!isFavorites"
          :playlist-id="isFavorites ? undefined : playlistId"
          @reorder="handleReorder"
          @navigate-album="(name) => router.push(`/albums/${encodeURIComponent(name)}`)"
          @navigate-artist="(name) => router.push(`/artists/${encodeURIComponent(name)}`)"
        />
      </div>
    </div>

    <ContextMenu :visible="contextMenu.visible.value" :x="contextMenu.x.value" :y="contextMenu.y.value"
      :items="contextMenu.items.value" @close="contextMenu.close()" />

    <CreatePlaylistDialog v-model:open="renameDialogOpen" :initial-name="playlist?.name" title="Rename Playlist"
      confirm-label="Save" @confirm="handleRename" />
    <ConfirmDialog v-model:open="deleteConfirmOpen" title="Delete Playlist"
      message="Delete this playlist? Tracks won't be removed from your library."
      confirm-label="Delete" :danger="true" @confirm="handleDelete" />
  </div>
</template>
