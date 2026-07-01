<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { Search, Disc, Music, User, PenTool } from 'lucide-vue-next'
import AlbumCard from '../components/AlbumCard.vue'
import ArtistCard from '../components/ArtistCard.vue'
import ComposerCard from '../components/ComposerCard.vue'
import SearchSection from '../components/SearchSection.vue'
import * as api from '../lib/invoke'

const router = useRouter()
const route = useRoute()

const query = ref((route.query.q as string) || '')
const searching = ref(false)
const results = ref<api.SearchAllResults>({ tracks: [], albums: [], artists: [], genres: [], composers: [] })

let debounceTimer: ReturnType<typeof setTimeout> | null = null

watch(query, () => {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(async () => {
    if (!query.value.trim()) {
      results.value = { tracks: [], albums: [], artists: [], genres: [], composers: [] }
      return
    }
    searching.value = true
    try {
      results.value = await api.searchAll(query.value.trim())
    } catch {}
    searching.value = false
  }, 300)
})

function navigateToAlbum(name: string) { router.push(`/albums/${encodeURIComponent(name)}`) }
function navigateToArtist(name: string) { router.push(`/artists/${encodeURIComponent(name)}`) }
function navigateToComposer(name: string) { router.push(`/composers/${encodeURIComponent(name)}`) }

const hasTracks = () => results.value.tracks.length > 0
const hasAlbums = () => results.value.albums.length > 0
const hasArtists = () => results.value.artists.length > 0
const hasComposers = () => results.value.composers.length > 0
const hasResults = () => hasTracks() || hasAlbums() || hasArtists() || hasComposers()
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">
    <div class="px-8 pt-8 pb-4 flex-shrink-0">
      <div class="relative max-w-xl">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-foreground opacity-50 pointer-events-none" />
        <input v-model="query" type="text" placeholder="Search tracks, artists, albums..."
          class="w-full pl-10 pr-4 py-2.5 text-sm rounded-xl bg-foreground/[0.04] border border-foreground/10 outline-none focus:border-primary/50 transition-colors"
          autofocus />
      </div>
    </div>

    <div class="flex-1 overflow-y-auto px-8 pb-8 space-y-12 custom-scrollbar">
      <div v-if="!query.trim()" class="flex flex-col items-center justify-center h-64 text-center">
        <div class="w-20 h-20 bg-foreground/[0.03] rounded-3xl flex items-center justify-center mb-6 ring-1 ring-foreground/[0.05]">
          <Search class="w-10 h-10 text-foreground opacity-30" />
        </div>
        <p class="text-foreground opacity-60 text-xl font-semibold">Search your music</p>
        <p class="text-foreground opacity-40 mt-2 max-w-xs">Find tracks, albums, artists, and more</p>
      </div>

      <div v-else-if="searching" class="space-y-12 mt-4">
        <div v-for="i in 2" :key="i" class="space-y-4">
          <div class="h-8 w-48 bg-foreground/[0.06] rounded-lg animate-pulse" />
          <div class="flex gap-6">
            <div v-for="j in 4" :key="j" class="w-48 aspect-square bg-foreground/[0.04] rounded-xl animate-pulse" />
          </div>
        </div>
      </div>

      <div v-else-if="query.trim() && !searching && !hasResults()"
        class="flex flex-col items-center justify-center h-64 text-center">
        <div class="w-20 h-20 bg-foreground/[0.03] rounded-3xl flex items-center justify-center mb-6 ring-1 ring-foreground/[0.05]">
          <Music class="w-10 h-10 text-foreground opacity-30" />
        </div>
        <p class="text-foreground opacity-60 text-xl font-semibold">No results found</p>
        <p class="text-foreground opacity-40 mt-2">Try a different search term</p>
      </div>

      <div v-else-if="hasResults()" class="space-y-16 py-4">
        <SearchSection v-if="hasTracks()" title="Tracks" :icon="Music"
          :items="results.tracks" id="search-tracks" :rows="3">
          <template #default="{ item: track }">
            <div class="flex items-center gap-3 p-2 rounded-xl hover:bg-foreground/[0.04] cursor-pointer group transition-all"
              @click="navigateToAlbum(track.album || '')">
              <div class="w-12 h-12 flex-shrink-0 rounded-lg bg-foreground/[0.06] overflow-hidden ring-1 ring-foreground/[0.06] flex items-center justify-center">
                <Music class="w-5 h-5 text-foreground opacity-40" />
              </div>
              <div class="min-w-0">
                <p class="text-sm font-semibold text-foreground truncate group-hover:text-primary transition-colors">{{ track.title }}</p>
                <p class="text-xs text-foreground opacity-60 truncate">{{ track.artist }}</p>
              </div>
            </div>
          </template>
        </SearchSection>

        <SearchSection v-if="hasAlbums()" title="Albums" :icon="Disc"
          :items="results.albums" id="search-albums">
          <template #default="{ item: album }">
            <AlbumCard :album="{ name: album.name, artist: album.artist, year: album.year, trackCount: album.track_count, firstTrackId: album.first_track_id }" @click="navigateToAlbum(album.name)" />
          </template>
        </SearchSection>

        <SearchSection v-if="hasArtists()" title="Artists" :icon="User"
          :items="results.artists" id="search-artists">
          <template #default="{ item: artist }">
            <ArtistCard :artist="{ id: artist as string, name: artist as string } as any" @click="navigateToArtist(artist as string)" />
          </template>
        </SearchSection>

        <SearchSection v-if="hasComposers()" title="Composers" :icon="PenTool"
          :items="results.composers" id="search-composers">
          <template #default="{ item: composer }">
            <ComposerCard :composer="{ id: composer as string, name: composer as string } as any" @click="navigateToComposer(composer as string)" />
          </template>
        </SearchSection>
      </div>
    </div>
  </div>
</template>
