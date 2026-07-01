<script setup lang="ts">
import { useRouter } from 'vue-router'
import { usePlayerStore } from '../stores/player'
import { useAlbumContextMenu } from '../composables/useAlbumContextMenu'
import { useContextMenu } from '../composables/useContextMenu'
import VirtualizedGrid from './VirtualizedGrid.vue'
import AlbumCard from './AlbumCard.vue'
import ContextMenu from './ContextMenu.vue'
import { useLibraryStore } from '../stores/library'

defineProps<{
  albums: { name: string; artist: string; year: number; trackCount: number; firstTrackId: string }[]
  gap?: number
}>()

const router = useRouter()
const playerStore = usePlayerStore()
const library = useLibraryStore()
const contextMenu = useContextMenu()
const { buildMenuItems } = useAlbumContextMenu()

function onContextMenu(e: MouseEvent, album: { name: string; artist: string; year: number; trackCount: number; firstTrackId: string }) {
  const tracks = library.tracks.filter(t => t.album === album.name)
  contextMenu.open(e, buildMenuItems(tracks))
}

function navigateToAlbum(name: string) { router.push(`/albums/${encodeURIComponent(name)}`) }
function navigateToArtist(artist: string) { if (artist) router.push(`/artists/${encodeURIComponent(artist)}`) }

function playAlbum(name: string) {
  const tracks = library.tracks.filter(t => t.album === name)
  if (tracks.length > 0) playerStore.playTracks(tracks, 0)
}
</script>

<template>
  <VirtualizedGrid :items="albums" :square-items="true" :text-area-height="60" :min-column-width="180" :gap="gap">
    <template #default="{ item: album }">
      <AlbumCard :album="album" @click="navigateToAlbum" @artist-click="navigateToArtist" @play="playAlbum"
        @contextmenu="onContextMenu" />
    </template>
  </VirtualizedGrid>
  <ContextMenu :visible="contextMenu.visible.value" :x="contextMenu.x.value" :y="contextMenu.y.value"
    :items="contextMenu.items.value" @close="contextMenu.close()" />
</template>
