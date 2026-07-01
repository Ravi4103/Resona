<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useLibraryStore } from '../stores/library'
import { usePlayerStore } from '../stores/player'
import ViewHeader from '../components/ViewHeader.vue'
import AlbumGrid from '../components/AlbumGrid.vue'
import AlbumListView from '../components/AlbumListView.vue'
import AlbumViewFilter from '../components/AlbumViewFilter.vue'

const router = useRouter()
const library = useLibraryStore()
const playerStore = usePlayerStore()

onMounted(() => { if (library.tracks.length === 0) library.loadAll() })

const searchQuery = ref('')
const viewMode = ref<'grid' | 'list'>(
  (localStorage.getItem('albums-view-mode') as 'grid' | 'list') || 'grid'
)
watch(viewMode, v => localStorage.setItem('albums-view-mode', v))

type SortCol = 'title' | 'artist' | 'year'
type SortDir = 'asc' | 'desc'
const sortColumn = ref<SortCol>(
  (localStorage.getItem('albums-sort-col') as SortCol) || 'title'
)
const sortDir = ref<SortDir>(
  (localStorage.getItem('albums-sort-dir') as SortDir) || 'asc'
)
watch(sortColumn, v => localStorage.setItem('albums-sort-col', v))
watch(sortDir, v => localStorage.setItem('albums-sort-dir', v))

const albums = computed(() => {
  const map = new Map<string, {
    name: string; artist: string; year: number; trackCount: number; firstTrackId: string
  }>()
  for (const t of library.tracks) {
    const key = t.album || 'Unknown Album'
    if (!map.has(key)) {
      map.set(key, { name: key, artist: t.album_artist || t.artist || '', year: t.year || 0, trackCount: 0, firstTrackId: t.id })
    }
    const entry = map.get(key)!
    entry.trackCount++
    if (!entry.artist) entry.artist = t.artist || ''
    if (!entry.year && t.year) entry.year = t.year
  }
  return Array.from(map.values())
})

function foldUnicode(s: string): string {
  return s.toLowerCase().normalize('NFD').replace(/[\u0300-\u036f]/g, '')
}

const processedAlbums = computed(() => {
  let result = albums.value
  if (searchQuery.value) {
    const q = foldUnicode(searchQuery.value)
    result = result.filter(a => foldUnicode(a.name).includes(q) || foldUnicode(a.artist).includes(q))
  }
  return [...result].sort((a, b) => {
    let cmp = 0
    if (sortColumn.value === 'title') cmp = a.name.localeCompare(b.name)
    else if (sortColumn.value === 'artist') cmp = a.artist.localeCompare(b.artist)
    else if (sortColumn.value === 'year') cmp = a.year - b.year
    return sortDir.value === 'asc' ? cmp : -cmp
  })
})

function navigateToAlbum(name: string) {
  const track = library.tracks.find(t => t.album === name)
  if (track) router.push(`/albums/${encodeURIComponent(name)}`)
}

function playAlbum(name: string) {
  const albumTracks = library.tracks.filter(t => t.album === name)
  if (albumTracks.length > 0) playerStore.playTracks(albumTracks, 0)
}

function navigateToArtist(name: string) {
  router.push(`/artists/${encodeURIComponent(name)}`)
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-background">
    <ViewHeader v-model="searchQuery" title="Albums" search-placeholder="Search albums...">
      <template #actions>
        <AlbumViewFilter
          :view-mode="viewMode"
          :sort-column="sortColumn"
          :sort-dir="sortDir"
          @update:view-mode="(v) => viewMode = v"
          @update:sort-column="(c) => sortColumn = c"
          @update:sort-dir="(d) => sortDir = d" />
      </template>
    </ViewHeader>

    <div class="flex-1 overflow-y-auto custom-scrollbar" :class="viewMode === 'grid' ? 'px-6 py-8' : ''">
      <div v-if="processedAlbums.length === 0" class="h-full flex items-center justify-center text-foreground opacity-50 text-sm">
        No albums found
      </div>
      <AlbumGrid v-else-if="viewMode === 'grid'" :albums="processedAlbums" @click="navigateToAlbum" @play="playAlbum"
        @artist-click="navigateToArtist" />
      <AlbumListView v-else :albums="processedAlbums" :sort-column="sortColumn" :sort-dir="sortDir"
        @click="navigateToAlbum" @play="playAlbum" @artist-click="navigateToArtist"
        @update:sort-column="(c: SortCol) => sortColumn = c"
        @update:sort-dir="(d: SortDir) => sortDir = d" />
    </div>
  </div>
</template>
