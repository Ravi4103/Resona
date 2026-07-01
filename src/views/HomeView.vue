<script setup lang="ts">
import { ref, shallowRef, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { Music, Settings as SettingsIcon, Sparkles, History, Ghost } from 'lucide-vue-next'
import { usePlayerStore } from '../stores/player'
import { useLibraryStore } from '../stores/library'
import { listen } from '@tauri-apps/api/event'
import * as api from '../lib/invoke'
import type { Track } from '../lib/invoke'
import TrackCard from '../components/TrackCard.vue'
import HomeSection from '../components/HomeSection.vue'
import TrackContextMenu from '../components/TrackContextMenu.vue'

const router = useRouter()
const playerStore = usePlayerStore()
const trackContextMenu = ref<InstanceType<typeof TrackContextMenu> | null>(null)

const library = useLibraryStore()
const recentlyPlayed = shallowRef<Track[]>([])
const mostListened = shallowRef<Track[]>([])
const leastListened = shallowRef<Track[]>([])
const loadingSections = ref(true)

const randomGreeting = computed(() => {
  const hour = new Date().getHours()
  if (hour < 12) return 'Good morning'
  if (hour < 17) return 'Good afternoon'
  if (hour < 21) return 'Good evening'
  return 'Good night'
})

const welcomePhrase = computed(() => {
  const phrases = ['What would you like to play?', 'Ready to dive in?', 'Discover something new']
  const seed = new Date().getHours()
  return phrases[seed % phrases.length]
})

async function loadSections() {
  loadingSections.value = true
  try {
    const [recent, most, least] = await Promise.all([
      api.getRecentlyPlayedTracks(28),
      api.getMostListenedTracks(28),
      api.getLeastListenedTracks(28),
    ])
    recentlyPlayed.value = recent || []
    mostListened.value = most || []
    leastListened.value = least || []
  } catch (err) {
    console.error('Failed to fetch home data:', err)
  } finally {
    loadingSections.value = false
  }
}

let offSyncFinished: (() => void) | null = null

onMounted(async () => {
  if (library.tracks.length === 0) await library.loadAll()
  await loadSections()
  offSyncFinished = await listen('library:sync-finished', () => {
    loadSections()
  })
})

onUnmounted(() => {
  offSyncFinished?.()
})

function playTrack(track: Track) {
  playerStore.playTracks([track], 0)
}

function playAll(tracks: Track[]) {
  if (tracks.length > 0) playerStore.playTracks(tracks, 0)
}

function navigateToSettings() {
  router.push('/settings/library')
}

function navigateToTrack(track: Track) {
  if (track.album) router.push(`/albums/${encodeURIComponent(track.album)}`)
}

function navigateToArtist(name: string) { router.push(`/artists/${encodeURIComponent(name)}`) }
function navigateToAlbum(name: string) { router.push(`/albums/${encodeURIComponent(name)}`) }

function onTrackContextMenu(e: MouseEvent, track: Track) {
  trackContextMenu.value?.open(e, track)
}
</script>

<template>
  <div class="p-8 h-full overflow-y-auto custom-scrollbar">
    <div v-if="library.loading" class="h-full flex items-center justify-center">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary" />
    </div>

    <div v-else-if="library.tracks.length === 0"
      class="h-full flex flex-col items-center justify-center text-center animate-in fade-in zoom-in duration-700">
      <div class="w-24 h-24 bg-foreground/5 rounded-3xl flex items-center justify-center mb-6 ring-1 ring-foreground/[0.06]">
        <Music class="w-12 h-12 text-foreground opacity-40" />
      </div>
      <h2 class="text-3xl font-bold mb-3">Your music awaits</h2>
      <p class="text-foreground opacity-60 max-w-md mb-8">Add your music folders in Settings to start listening</p>
      <button @click="navigateToSettings"
        class="flex items-center gap-2 px-6 py-3 bg-primary text-primary-foreground rounded-lg font-medium hover:scale-105 transition-transform shadow-lg shadow-primary/20">
        <SettingsIcon class="w-4 h-4" />
        Add Music Folder
      </button>
    </div>

    <div v-else class="space-y-16 pb-12 animate-in fade-in duration-700">
      <header class="select-none">
        <h1 class="text-4xl font-bold tracking-tight mb-2">{{ randomGreeting }}</h1>
        <p class="text-xl text-foreground opacity-60">{{ welcomePhrase }}</p>
      </header>

      <div v-if="loadingSections" class="flex items-center justify-center py-12">
        <div class="animate-spin rounded-full h-8 w-8 border-2 border-primary border-t-transparent" />
      </div>

      <template v-else>
        <HomeSection title="Keep Listening" :icon="History" :items="recentlyPlayed" id="carousel-recent"
          @play-all="playAll(recentlyPlayed)">
          <template #default="{ item: track }">
            <TrackCard :track="track" @play="playTrack" @click="navigateToTrack"
              @artist-click="navigateToArtist" @album-click="navigateToAlbum"
              @contextmenu="onTrackContextMenu" />
          </template>
        </HomeSection>

        <HomeSection title="Smart Mix" :icon="Sparkles" :items="mostListened" id="carousel-most"
          @play-all="playAll(mostListened)">
          <template #default="{ item: track }">
            <TrackCard :track="track" @play="playTrack" @click="navigateToTrack"
              @artist-click="navigateToArtist" @album-click="navigateToAlbum"
              @contextmenu="onTrackContextMenu" />
          </template>
        </HomeSection>

        <HomeSection title="Forgotten Gems" :icon="Ghost" :items="leastListened" id="carousel-least"
          @play-all="playAll(leastListened)">
          <template #default="{ item: track }">
            <TrackCard :track="track" @play="playTrack" @click="navigateToTrack"
              @artist-click="navigateToArtist" @album-click="navigateToAlbum"
              @contextmenu="onTrackContextMenu" />
          </template>
        </HomeSection>
      </template>
    </div>
  </div>

  <TrackContextMenu ref="trackContextMenu" />
</template>
