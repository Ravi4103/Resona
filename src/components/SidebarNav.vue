<script setup lang="ts">
import { Disc, Home, Music, Users, ListMusic, PenTool } from 'lucide-vue-next'
import { useLibraryStore } from '../stores/library'
import { usePlayerStore } from '../stores/player'
import SidebarItem from './SidebarItem.vue'

const library = useLibraryStore()
const playerStore = usePlayerStore()

const navItems = [
  { name: 'Home', icon: Home, to: '/' },
  { name: 'Albums', icon: Disc, to: '/albums' },
  { name: 'Artists', icon: Users, to: '/artists' },
  { name: 'Composers', icon: PenTool, to: '/composers' },
  { name: 'Tracks', icon: Music, to: '/tracks', shuffleOnDblClick: true },
  { name: 'Genres', icon: ListMusic, to: '/genres' },
]

function shuffleAllTracks() {
  const tracks = [...library.tracks]
  for (let i = tracks.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [tracks[i], tracks[j]] = [tracks[j], tracks[i]]
  }
  if (tracks.length > 0) {
    playerStore.setShuffle(true)
    playerStore.playTracks(tracks, 0)
  }
}
</script>

<template>
  <nav class="px-3 py-2 space-y-0.5">
    <SidebarItem
      v-for="item in navItems"
      :key="item.name"
      :to="item.to"
      :icon="item.icon"
      :label="item.name"
      @dblclick="item.shuffleOnDblClick ? shuffleAllTracks() : undefined" />
  </nav>
</template>
