<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { Tag, Play, Shuffle, MoreVertical } from 'lucide-vue-next'
import { usePlayerStore } from '../stores/player'
import DetailHero from '../components/DetailHero.vue'
import GenreDetailInfo from '../components/GenreDetailInfo.vue'
import GroupedAlbumList from '../components/GroupedAlbumList.vue'
import TrackTableSection from '../components/TrackTableSection.vue'
import ContextMenu from '../components/ContextMenu.vue'
import { useContextMenu } from '../composables/useContextMenu'
import { useGroupContextMenu } from '../composables/useGroupContextMenu'
import * as api from '../lib/invoke'

const route = useRoute()
const playerStore = usePlayerStore()
const contextMenu = useContextMenu()
const { buildMenuItems } = useGroupContextMenu()

const genreName = ref('')
const tracks = ref<api.Track[]>([])

async function load() {
  const genreId = decodeURIComponent(route.params.id as string)
  try {
    const [genreTracks, allGenres] = await Promise.all([
      api.getTracksByGenreId(genreId),
      api.getAllGenres()
    ])
    tracks.value = genreTracks
    genreName.value = allGenres.find(g => g.id === genreId)?.name || genreId
  } catch (e) {
    console.error('Failed to load genre detail:', e)
  }
}

onMounted(load)

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

function openContextMenu(e: MouseEvent) {
  contextMenu.open(e, buildMenuItems(tracks.value, playAll, shufflePlay))
}
</script>

<template>
  <div class="h-full flex flex-col bg-background overflow-hidden animate-in fade-in slide-in-from-right-4 duration-300">
    <DetailHero :title="genreName" @contextmenu.prevent="openContextMenu">
      <template #artwork>
        <div class="w-24 h-24 rounded-2xl bg-foreground/5 flex items-center justify-center ring-1 ring-foreground/[0.08] flex-shrink-0">
          <Tag class="w-12 h-12 text-foreground opacity-70" />
        </div>
      </template>
      <template #metadata>
        <GenreDetailInfo :tracks="tracks" />
      </template>
      <template #actions>
        <button @click="playAll"
          class="flex items-center gap-2 px-5 py-2.5 bg-primary text-primary-foreground rounded-full font-medium hover:scale-105 transition-transform shadow-lg shadow-primary/20">
          <Play class="w-5 h-5 fill-current" /> Play
        </button>
        <div class="flex gap-2">
          <button @click="shufflePlay"
            class="flex items-center gap-2 px-5 py-2.5 bg-foreground/10 text-foreground rounded-full font-medium hover:bg-foreground/20 transition-all">
            <Shuffle class="w-4 h-4" /> Shuffle
          </button>
          <button @click="openContextMenu"
            class="flex items-center gap-2 px-3 py-2.5 bg-foreground/10 text-foreground rounded-full font-medium hover:bg-foreground/20 transition-all">
            <MoreVertical class="w-4 h-4" />
          </button>
        </div>
      </template>
    </DetailHero>
    <div class="flex-1 overflow-y-auto custom-scrollbar">
      <div class="p-6">
        <TrackTableSection>
          <GroupedAlbumList :tracks="tracks" />
        </TrackTableSection>
      </div>
    </div>

    <ContextMenu :visible="contextMenu.visible.value" :x="contextMenu.x.value" :y="contextMenu.y.value"
      :items="contextMenu.items.value" @close="contextMenu.close()" />
  </div>
</template>
