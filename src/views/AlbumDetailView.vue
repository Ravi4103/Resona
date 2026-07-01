<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useLibraryStore } from '../stores/library'
import { usePlayerStore } from '../stores/player'
import TrackTable from '../components/TrackTable.vue'
import TrackTableSection from '../components/TrackTableSection.vue'
import AlbumDetailHero from '../components/AlbumDetailHero.vue'
import AlbumDetailInfo from '../components/AlbumDetailInfo.vue'
import ContextMenu from '../components/ContextMenu.vue'
import AppendToQueueDialog from '../components/AppendToQueueDialog.vue'
import AddToPlaylistDialog from '../components/AddToPlaylistDialog.vue'
import { useAlbumContextMenu } from '../composables/useAlbumContextMenu'
import { useContextMenu } from '../composables/useContextMenu'
import * as api from '../lib/invoke'

const route = useRoute()
const router = useRouter()
const library = useLibraryStore()
const playerStore = usePlayerStore()

const album = computed(() => decodeURIComponent(route.params.id as string))
const tracks = computed(() => library.tracks.filter(t => (t.album || '') === album.value))

const artist = computed(() => tracks.value[0]?.album_artist || tracks.value[0]?.artist || 'Unknown Artist')
const artworkSrc = ref<string | null>(null)

onMounted(async () => {
  const first = tracks.value[0]
  if (first?.has_artwork) {
    try {
      const b64 = await api.getArtwork(first.id)
      if (b64) artworkSrc.value = `data:image/jpeg;base64,${b64}`
    } catch {}
  }
})

const contextMenu = useContextMenu()
const { buildMenuItems } = useAlbumContextMenu()
function openContextMenu(e: MouseEvent) {
  const items = buildMenuItems(tracks.value)
  contextMenu.open(e, items)
}

const queueDialogOpen = ref(false)
const playlistDialogOpen = ref(false)

function playAll() {
  if (tracks.value.length > 0) playerStore.playTracks(tracks.value, 0)
}

function shufflePlay() {
  const shuffled = [...tracks.value]
  for (let i = shuffled.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]]
  }
  if (shuffled.length > 0) playerStore.playTracks(shuffled, 0)
}

function addToQueue(position: 'next' | 'end') {
  const q = [...playerStore.queue]
  if (position === 'next') {
    q.splice(playerStore.queueIndex + 1, 0, ...tracks.value)
  } else {
    q.push(...tracks.value)
  }
  playerStore.queue = q
  queueDialogOpen.value = false
}

async function addToPlaylist(playlistId: string) {
  const trackIds = tracks.value.map(t => t.id)
  await api.addTracksToPlaylist(playlistId, trackIds)
  playlistDialogOpen.value = false
}

async function handleCreatePlaylist(name: string) {
  const pl = await library.createPlaylist(name)
  if (pl) {
    const trackIds = tracks.value.map(t => t.id)
    await api.addTracksToPlaylist(pl.id, trackIds)
  }
  playlistDialogOpen.value = false
}
</script>

<template>
  <div class="h-full flex flex-col bg-background overflow-hidden">
    <div class="flex-1 overflow-y-auto custom-scrollbar">
      <AlbumDetailHero
        :title="album"
        :artist="artist"
        :artwork-src="artworkSrc"
        :track-count="tracks.length"
        @play="playAll"
        @shuffle="shufflePlay"
        @contextmenu="openContextMenu"
        @artist-click="(name) => router.push(`/artists/${encodeURIComponent(name)}`)">
        <template #metadata>
          <AlbumDetailInfo :tracks="tracks" />
        </template>
      </AlbumDetailHero>

      <div class="p-6 pb-12">
        <TrackTableSection>
          <TrackTable
            :tracks="tracks"
            :show-artwork="false"
            :simple-mode="true"
            @navigate-album="(name) => router.push(`/albums/${encodeURIComponent(name)}`)"
            @navigate-artist="(name) => router.push(`/artists/${encodeURIComponent(name)}`)" />
        </TrackTableSection>
      </div>
    </div>

    <ContextMenu :visible="contextMenu.visible.value" :x="contextMenu.x.value" :y="contextMenu.y.value"
      :items="contextMenu.items.value" @close="contextMenu.close()" />

    <AppendToQueueDialog :open="queueDialogOpen" :track-count="tracks.length"
      @confirm="addToQueue" @close="queueDialogOpen = false" />

    <AddToPlaylistDialog :open="playlistDialogOpen" :track-count="tracks.length"
      :playlists="library.playlists"
      @confirm="addToPlaylist" @create-playlist="handleCreatePlaylist" @close="playlistDialogOpen = false" />
  </div>
</template>
