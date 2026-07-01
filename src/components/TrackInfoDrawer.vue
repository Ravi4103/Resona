<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { Music, AudioLines, X, Disc, User, Tag, Calendar, Clock, HardDrive, PenTool } from 'lucide-vue-next'
import { formatTime } from '../lib/utils'
import { usePlayerStore } from '../stores/player'
import * as api from '../lib/invoke'

const store = usePlayerStore()

const track = computed(() => store.trackInfoTrack)
const artworkSrc = ref<string | null>(null)

async function loadArtwork(id: string) {
  artworkSrc.value = null
  if (id && track.value?.has_artwork) {
    try {
      const b64 = await api.getArtwork(id)
      if (b64) artworkSrc.value = `data:image/jpeg;base64,${b64}`
    } catch {}
  }
}

watch(() => track.value?.id, async (id) => {
  if (id) loadArtwork(id)
}, { immediate: true })

async function changeArtwork() {
  const id = track.value?.id
  if (!id) return
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = 'image/jpeg,image/png,image/webp'
  input.onchange = async () => {
    const file = input.files?.[0]
    if (!file) return
    const reader = new FileReader()
    reader.onload = async () => {
      const b64 = (reader.result as string).split(',')[1]
      try {
        await api.setArtwork(id, b64)
        if (track.value) track.value.has_artwork = true
        loadArtwork(id)
      } catch (e) {
        console.error('Failed to set artwork', e)
      }
    }
    reader.readAsDataURL(file)
  }
  input.click()
}

const details = computed(() => {
  const t = track.value
  if (!t) return []
  return [
    { label: 'Album', value: t.album || '-', icon: Disc },
    { label: 'Artist', value: t.artist || '-', icon: User },
    { label: 'Composer', value: t.composer || '-', icon: PenTool },
    { label: 'Genre', value: t.raw_genre_names || t.genre || '-', icon: Tag },
    { label: 'Year', value: t.year ? t.year.toString() : '-', icon: Calendar },
    { label: 'Duration', value: t.duration ? formatTime(t.duration) : '-', icon: Clock },
    { label: 'Format', value: t.file_format ? t.file_format.toUpperCase() : '-', icon: Music },
    { label: 'Sample Rate', value: t.sample_rate ? `${(t.sample_rate / 1000).toFixed(1)} kHz` : '-', icon: AudioLines },
    { label: 'Bit Depth', value: t.bit_depth ? `${t.bit_depth}-bit` : '-', icon: AudioLines },
    { label: 'Track', value: t.track_number ? `${t.track_number}${t.disc_number > 1 ? ` (Disc ${t.disc_number})` : ''}` : '-', icon: Music },
    { label: 'File Size', value: t.file_size ? `${(t.file_size / 1024 / 1024).toFixed(1)} MB` : '-', icon: HardDrive },
    { label: 'Path', value: t.file_path || '-', icon: HardDrive, mono: true },
  ]
})
</script>

<template>
  <div class="h-full flex flex-col bg-background/80 backdrop-blur-2xl text-foreground">
    <div class="flex items-center justify-between px-4 py-3 border-b border-foreground/[0.06] select-none">
      <div class="flex items-center gap-2 font-semibold">
        <AudioLines class="w-4 h-4 text-primary" />
        <span class="text-sm">Track Info</span>
      </div>
      <button class="p-1.5 rounded-full hover:bg-foreground/10 transition-colors text-foreground opacity-60 hover:opacity-100"
        @click="store.toggleTrackInfoDrawer()">
        <X class="w-4 h-4" />
      </button>
    </div>

    <div class="flex-1 overflow-y-auto custom-scrollbar">
      <div v-if="track" class="flex flex-col">
        <!-- Artwork -->
        <div class="p-6 flex flex-col items-center gap-4 bg-gradient-to-b from-dynamic-surface/50 to-transparent">
          <div class="w-44 h-44 rounded-2xl overflow-hidden shadow-2xl ring-1 ring-foreground/10 flex-shrink-0 relative group cursor-pointer"
            @click="changeArtwork">
            <img v-if="artworkSrc" :src="artworkSrc" :alt="track.title" class="w-full h-full object-cover" />
            <div v-else class="w-full h-full bg-foreground/5 flex items-center justify-center">
              <Music class="w-16 h-16 text-foreground opacity-30" />
            </div>
            <div class="absolute inset-0 bg-black/40 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
              <span class="text-white text-xs font-medium">Change Artwork</span>
            </div>
          </div>
          <div class="text-center w-full px-2">
            <h2 class="text-lg font-bold truncate max-w-full">{{ track.title || 'Unknown' }}</h2>
            <p class="text-sm text-foreground opacity-60 truncate max-w-full mt-0.5">{{ track.artist || 'Unknown Artist' }}</p>
            <p v-if="track.album" class="text-xs text-foreground opacity-40 truncate max-w-full mt-0.5">{{ track.album }}</p>
          </div>
        </div>

        <!-- Details -->
        <div class="px-4 py-2 space-y-0.5">
          <div v-for="d in details" :key="d.label"
            class="flex justify-between items-start gap-4 py-2.5 border-b border-foreground/[0.04] last:border-0">
            <div class="flex items-center gap-2 text-foreground opacity-50 shrink-0">
              <component :is="d.icon" class="w-3.5 h-3.5" />
              <span class="text-xs">{{ d.label }}</span>
            </div>
            <span class="text-xs text-right max-w-[55%] break-words" :class="d.mono ? 'font-mono text-[10px] text-foreground opacity-40' : ''">{{ d.value }}</span>
          </div>
        </div>
      </div>
      <div v-else class="flex flex-col items-center justify-center h-full text-foreground opacity-50 gap-3 py-20">
        <AudioLines class="w-10 h-10 opacity-20" />
        <p class="text-sm">No track selected</p>
      </div>
    </div>
  </div>
</template>
