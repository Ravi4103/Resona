<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useLibraryStore } from '../stores/library'
import { usePlayerStore } from '../stores/player'
import DetailHero from '../components/DetailHero.vue'
import ComposerDetailInfo from '../components/ComposerDetailInfo.vue'
import GroupedAlbumList from '../components/GroupedAlbumList.vue'
import TrackTableSection from '../components/TrackTableSection.vue'
import ContextMenu from '../components/ContextMenu.vue'
import { PenTool, Play, Shuffle, MoreVertical } from 'lucide-vue-next'
import { useContextMenu } from '../composables/useContextMenu'
import { useGroupContextMenu } from '../composables/useGroupContextMenu'

const route = useRoute()
const library = useLibraryStore()
const playerStore = usePlayerStore()
const contextMenu = useContextMenu()
const { buildMenuItems } = useGroupContextMenu()

const composer = computed(() => decodeURIComponent(route.params.id as string))

const tracks = computed(() =>
  library.tracks.filter(t => t.composer?.trim() === composer.value)
)

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
    <DetailHero :title="composer" @contextmenu.prevent="openContextMenu">
      <template #artwork>
        <div class="w-32 h-32 xl:w-40 xl:h-40 rounded-full shadow-2xl overflow-hidden ring-2 ring-foreground/[0.08] bg-foreground/5 flex-shrink-0 flex items-center justify-center">
          <PenTool class="w-16 h-16 text-foreground opacity-40" />
        </div>
      </template>
      <template #metadata>
        <ComposerDetailInfo :tracks="tracks" />
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
