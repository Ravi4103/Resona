<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useLibraryStore } from '../stores/library'
import ViewHeader from '../components/ViewHeader.vue'
import TrackTable from '../components/TrackTable.vue'

const library = useLibraryStore()

onMounted(() => { if (library.tracks.length === 0) library.loadAll() })

const searchQuery = ref('')

function foldUnicode(s: string): string {
  return s.toLowerCase().normalize('NFD').replace(/[\u0300-\u036f]/g, '')
}

const filteredTracks = computed(() => {
  if (!searchQuery.value) return library.tracks
  const q = foldUnicode(searchQuery.value)
  return library.tracks.filter(t =>
    foldUnicode(t.title).includes(q) ||
    foldUnicode(t.artist || '').includes(q) ||
    foldUnicode(t.album || '').includes(q)
  )
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-background">
    <ViewHeader v-model="searchQuery" title="Tracks" search-placeholder="Search tracks..." />
    <TrackTable :tracks="filteredTracks" :is-loading="library.loading" />
  </div>
</template>
