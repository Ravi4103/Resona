<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { Play, Disc } from 'lucide-vue-next'
import type { Track } from '../lib/invoke'
import { usePlayerStore } from '../stores/player'
import { useTrackTableSettings } from '../composables/useTrackTableSettings'
import * as api from '../lib/invoke'
import TrackTable from './TrackTable.vue'

const playerStore = usePlayerStore()
const router = useRouter()
const settings = useTrackTableSettings()

const emit = defineEmits<{
  playTrack: [track: Track, index: number]
  navigateAlbum: [name: string]
  navigateArtist: [name: string]
}>()

const { tracks } = defineProps<{ tracks: Track[] }>()

const TABLE_HEADER_HEIGHT = 41

function tableHeight(trackCount: number): string {
  const rowHeight = settings.collapsedMode.value ? 36 : 56
  return `${trackCount * rowHeight + TABLE_HEADER_HEIGHT}px`
}

const artworkCache = ref<Record<string, string>>({})

// Load artwork for album headers
watch(() => tracks, async (val) => {
  const grouped = new Map<string, Track[]>()
  for (const t of val) {
    const album = t.album || 'Unknown Album'
    if (!grouped.has(album)) grouped.set(album, [])
    grouped.get(album)!.push(t)
  }
  for (const [, groupTracks] of grouped) {
    const first = groupTracks[0]
    if (first?.has_artwork && !artworkCache.value[first.id]) {
      try {
        const b64 = await api.getArtwork(first.id)
        if (b64) artworkCache.value[first.id] = `data:image/jpeg;base64,${b64}`
      } catch {}
    }
  }
}, { immediate: true })

const groupedAlbums = computed(() => {
  const groups: Record<string, { name: string; artist: string; year: number; tracks: Track[] }> = {}

  for (const track of tracks) {
    const name = track.album || 'Unknown Album'
    if (!groups[name]) {
      groups[name] = { name, artist: '', year: 0, tracks: [] }
    }
    const g = groups[name]
    g.tracks.push(track)
    if (track.album_artist || track.artist) g.artist = track.album_artist || track.artist
    if (track.year) g.year = track.year
  }

  const result = Object.values(groups)

  result.sort((a, b) => {
    if (a.name === 'Unknown Album') return 1
    if (b.name === 'Unknown Album') return -1
    if (a.year !== b.year) return b.year - a.year
    return a.name.localeCompare(b.name)
  })

  for (const group of result) {
    group.tracks.sort((t1, t2) => {
      const d1 = t1.disc_number || 1
      const d2 = t2.disc_number || 1
      if (d1 !== d2) return d1 - d2
      return (t1.track_number || 0) - (t2.track_number || 0)
    })
  }

  return result
})

function playGroup(group: { name: string; tracks: Track[] }) {
  playerStore.playTracks(group.tracks, 0)
}
</script>

<template>
  <div class="space-y-12 pb-12">
    <div v-for="group in groupedAlbums" :key="group.name" class="space-y-4">
      <!-- Album Header -->
      <div class="flex items-end gap-6 pr-2">
        <div
          class="w-32 h-32 md:w-40 md:h-40 rounded-xl shadow-xl overflow-hidden ring-1 ring-foreground/10 bg-foreground/5 flex-shrink-0 group relative cursor-pointer"
          @click="router.push(`/albums/${encodeURIComponent(group.name)}`)">
          <img v-if="artworkCache[group.tracks[0]?.id]" :src="artworkCache[group.tracks[0].id]"
            :alt="group.name" class="w-full h-full object-cover" />
          <div v-else class="w-full h-full flex items-center justify-center text-foreground opacity-30">
            <Disc class="w-16 h-16" />
          </div>
          <div
            class="absolute inset-0 bg-background/30 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
            <button
              class="w-12 h-12 bg-foreground text-background rounded-full shadow-xl flex items-center justify-center transform scale-90 group-hover:scale-100 transition-all duration-300"
              @click.stop="playGroup(group)">
              <Play class="w-6 h-6 fill-current ml-1" />
            </button>
          </div>
        </div>

        <div class="flex-1 pb-2">
          <h2 class="text-2xl md:text-3xl font-bold tracking-tight mb-1 cursor-pointer hover:text-primary transition-colors inline-block"
            @click="router.push(`/albums/${encodeURIComponent(group.name)}`)">
            {{ group.name }}
          </h2>
          <div class="flex items-center gap-3 text-sm text-foreground opacity-60">
            <span v-if="group.year" class="font-medium">{{ group.year }}</span>
            <span v-if="group.year">•</span>
            <span>{{ group.tracks.length }} tracks</span>
          </div>
        </div>
      </div>

      <!-- Track Table -->
      <div class="rounded-xl overflow-hidden ring-1 ring-foreground/[0.06]"
        :style="{ height: tableHeight(group.tracks.length) }">
        <TrackTable :tracks="group.tracks" :simple-mode="true"
          @play-track="(t, i) => emit('playTrack', t, i)"
          @navigate-album="(n) => emit('navigateAlbum', n)"
          @navigate-artist="(n) => emit('navigateArtist', n)" />
      </div>
    </div>
  </div>
</template>
