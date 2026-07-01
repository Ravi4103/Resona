<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Disc, Play, User } from 'lucide-vue-next'
import * as api from '../lib/invoke'

const props = withDefaults(defineProps<{
  album: { name: string; artist: string; year: number; trackCount: number; firstTrackId: string }
  showPlay?: boolean
}>(), { showPlay: true })

const emit = defineEmits<{
  click: [name: string]
  play: [name: string]
  artistClick: [artist: string]
  contextmenu: [e: MouseEvent, album: { name: string; artist: string; year: number; trackCount: number; firstTrackId: string }]
}>()

const artworkSrc = ref<string | null>(null)
const imgRef = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

onMounted(() => {
  observer = new IntersectionObserver((entries) => {
    entries.forEach(async (entry) => {
      if (entry.isIntersecting) {
        try { const b64 = await api.getArtwork(props.album.firstTrackId); if (b64) artworkSrc.value = `data:image/jpeg;base64,${b64}` } catch {}
        if (observer && imgRef.value) { observer.unobserve(imgRef.value); observer.disconnect(); observer = null }
      }
    })
  }, { rootMargin: '200px' })
  if (imgRef.value) observer.observe(imgRef.value)
})

onUnmounted(() => { if (observer && imgRef.value) { observer.unobserve(imgRef.value); observer.disconnect() } })
</script>

<template>
  <div ref="imgRef" class="group cursor-pointer" @click="emit('click', album.name)" @contextmenu.prevent="emit('contextmenu', $event, album)">
    <div class="aspect-square bg-foreground/5 rounded-lg ring-1 ring-foreground/[0.06] overflow-hidden relative mb-3 transition-all">
      <img v-if="artworkSrc" :src="artworkSrc" :alt="album.name"
        class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" />
      <div v-else class="w-full h-full flex items-center justify-center text-foreground opacity-40 group-hover:scale-105 transition-transform duration-500">
        <Disc class="w-1/3 h-1/3" />
      </div>
      <div v-if="showPlay" class="absolute inset-0 bg-background/20 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
        <button @click.stop="emit('play', album.name)"
          class="w-12 h-12 bg-foreground text-background rounded-full shadow-xl flex items-center justify-center transform translate-y-4 group-hover:translate-y-0 transition-all duration-300">
          <Play class="w-6 h-6 fill-current ml-1" />
        </button>
      </div>
    </div>
    <div class="space-y-1 px-1">
      <h3 class="font-medium text-sm truncate group-hover:text-foreground transition-colors">{{ album.name || 'Unknown Album' }}</h3>
      <div class="text-xs text-foreground opacity-60 truncate flex items-center gap-1">
        <User class="w-3 h-3 flex-shrink-0" />
        <span class="truncate hover:text-primary cursor-pointer transition-colors" @click.stop="emit('artistClick', album.artist || '')">{{ album.artist || 'Unknown Artist' }}</span>
      </div>
    </div>
  </div>
</template>
