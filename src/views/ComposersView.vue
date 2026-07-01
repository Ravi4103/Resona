<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { PenTool } from 'lucide-vue-next'
import { useLibraryStore } from '../stores/library'
import { usePlayerStore } from '../stores/player'
import EntityExplorerLayout from '../components/EntityExplorerLayout.vue'

const router = useRouter()
const route = useRoute()
const library = useLibraryStore()
const playerStore = usePlayerStore()

const composers = computed(() => {
  const map = new Map<string, { id: string; name: string; trackCount: number }>()
  for (const t of library.tracks) {
    const name = t.composer?.trim()
    if (!name) continue
    if (!map.has(name)) map.set(name, { id: name, name, trackCount: 0 })
    map.get(name)!.trackCount++
  }
  return Array.from(map.values()).sort((a, b) => a.name.localeCompare(b.name))
})

const isLoading = computed(() => library.loading)

function onSelect(id: string) {
  router.push(`/composers/${encodeURIComponent(id)}`)
}

function onPlay(item: { id: string; name: string }) {
  const tracks = library.tracks.filter(t => t.composer?.trim() === item.name)
  if (tracks.length > 0) playerStore.playTracks(tracks, 0)
}
</script>

<template>
  <EntityExplorerLayout title="Composers" :items="composers" :is-loading="isLoading"
    :selected-id="(route.params.id as string)" :icon="PenTool"
    search-placeholder="Search composers..."
    @select="onSelect" @play="onPlay">
    <router-view v-slot="{ Component }">
      <KeepAlive :max="5">
        <component :is="Component" :key="route.params.id" />
      </KeepAlive>
    </router-view>
  </EntityExplorerLayout>
</template>
