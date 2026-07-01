<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Music, Play, User, Disc } from 'lucide-vue-next'
import type { Track } from '../lib/invoke'
import * as api from '../lib/invoke'

const props = defineProps<{
  track: Track
}>()

const emit = defineEmits<{
  click: [track: Track]
  play: [track: Track]
  'artist-click': [name: string]
  'album-click': [name: string]
  contextmenu: [e: MouseEvent, track: Track]
}>()

const cardRef = ref<HTMLElement | null>(null)
const artworkSrc = ref<string | null>(null)
let objectUrl: string | null = null
let observer: IntersectionObserver | null = null

async function loadArtwork() {
  try {
    const bytes = await api.getTrackArtwork(props.track.id, 'md')
    if (bytes) {
      const blob = new Blob([new Uint8Array(bytes)], { type: 'image/jpeg' })
      objectUrl = URL.createObjectURL(blob)
      artworkSrc.value = objectUrl
    }
  } catch {}
}

onMounted(() => {
  observer = new IntersectionObserver((entries) => {
    entries.forEach(async (entry) => {
      if (entry.isIntersecting) {
        await loadArtwork()
        if (observer && cardRef.value) {
          observer.unobserve(cardRef.value)
          observer.disconnect()
          observer = null
        }
      }
    })
  }, { rootMargin: '150px' })
  if (cardRef.value) {
    observer.observe(cardRef.value)
  }
})

onUnmounted(() => {
  if (observer && cardRef.value) {
    observer.unobserve(cardRef.value)
    observer.disconnect()
  }
  if (objectUrl) {
    URL.revokeObjectURL(objectUrl)
  }
})
</script>

<template>
  <div ref="cardRef" class="group cursor-pointer w-full" @click="emit('click', track)"
    @contextmenu.prevent="emit('contextmenu', $event, track)">
    <div class="aspect-square bg-foreground/5 rounded-lg ring-1 ring-foreground/[0.06] overflow-hidden relative mb-3 transition-all">
      <img v-if="artworkSrc" :src="artworkSrc" :alt="track.title"
        class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" />
      <div v-else class="w-full h-full flex items-center justify-center text-foreground opacity-40 group-hover:scale-105 transition-transform duration-500">
        <Music class="w-1/3 h-1/3" />
      </div>
      <div class="absolute inset-0 bg-background/20 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
        <button @click.stop="emit('play', track)"
          class="w-10 h-10 bg-foreground text-background rounded-full shadow-xl flex items-center justify-center transform translate-y-4 group-hover:translate-y-0 transition-all duration-300">
          <Play class="w-5 h-5 fill-current ml-1" />
        </button>
      </div>
    </div>
    <div class="space-y-1 px-1">
      <h3 class="font-medium text-sm truncate group-hover:text-foreground transition-colors">{{ track.title || 'Unknown' }}</h3>
      <div class="text-xs text-foreground opacity-60 truncate flex flex-col gap-0.5">
        <div class="flex items-center gap-1 truncate">
          <User class="w-3 h-3 flex-shrink-0" />
          <span class="truncate hover:text-primary cursor-pointer transition-colors"
            @click.stop="emit('artist-click', track.artist || 'Unknown Artist')">{{ track.artist || 'Unknown Artist' }}</span>
        </div>
        <div v-if="track.album" class="flex items-center gap-1 truncate opacity-80">
          <Disc class="w-3 h-3 flex-shrink-0" />
          <span class="truncate hover:text-primary cursor-pointer transition-colors"
            @click.stop="emit('album-click', track.album)">{{ track.album }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
