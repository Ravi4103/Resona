<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { Tag } from 'lucide-vue-next'
import { usePlayerStore } from '../stores/player'
import { listen } from '@tauri-apps/api/event'
import * as api from '../lib/invoke'
import EntityExplorerLayout from '../components/EntityExplorerLayout.vue'

const router = useRouter()
const route = useRoute()
const playerStore = usePlayerStore()

const genres = ref<{ id: string; name: string; trackCount?: number }[]>([])
const isLoading = ref(true)

async function load() {
  isLoading.value = true
  try {
    const result = await api.getAllGenres()
    genres.value = result.map(g => ({ id: g.id, name: g.name, trackCount: g.track_count }))
  } catch (e) {
    console.error('Failed to load genres:', e)
  }
  isLoading.value = false
}

onMounted(() => {
  load()
  const unlisten = listen('library:sync-finished', () => load())
  return () => { unlisten.then(fn => fn()) }
})

function onSelect(id: string) {
  router.push(`/genres/${encodeURIComponent(id)}`)
}

async function onPlay(item: { id: string; name: string }) {
  try {
    const tracks = await api.getTracksByGenreId(item.id)
    if (tracks.length > 0) playerStore.playTracks(tracks, 0)
  } catch (e) {
    console.error('Failed to play genre tracks:', e)
  }
}
</script>

<template>
  <EntityExplorerLayout title="Genres" :items="genres" :is-loading="isLoading"
    :selected-id="(route.params.id as string)" :icon="Tag"
    search-placeholder="Search genres..."
    @select="onSelect" @play="onPlay">
    <router-view v-slot="{ Component }">
      <KeepAlive :max="5">
        <component :is="Component" :key="route.params.id" />
      </KeepAlive>
    </router-view>
  </EntityExplorerLayout>
</template>

