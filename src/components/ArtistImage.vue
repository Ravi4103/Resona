<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { User } from 'lucide-vue-next'
import { listen } from '@tauri-apps/api/event'
import * as api from '../lib/invoke'
import { useAppStore } from '../stores/app'

const props = defineProps<{
  artist: string
  size?: string
}>()

const appStore = useAppStore()
const blobUrl = ref<string | null>(null)
const loading = ref(false)
const error = ref(false)
let unlisten: (() => void) | null = null

async function load() {
  if (!props.artist || !appStore.useOnlineArtistArtwork) {
    blobUrl.value = null
    return
  }
  loading.value = true
  error.value = false
  try {
    const bytes = await api.getArtistArtwork(props.artist, props.size)
    if (bytes && bytes.length > 0) {
      const blob = new Blob([new Uint8Array(bytes)], { type: 'image/jpeg' })
      blobUrl.value = URL.createObjectURL(blob)
    } else {
      blobUrl.value = null
    }
  } catch {
    error.value = true
  }
  loading.value = false
}

onMounted(async () => {
  await load()
  if (appStore.useOnlineArtistArtwork) {
    unlisten = await listen<{ artist: string; key: string }>('artist-artwork-updated', async (event) => {
      if (event.payload.artist === props.artist) {
        if (blobUrl.value) URL.revokeObjectURL(blobUrl.value)
        await load()
      }
    })
  }
})

onUnmounted(() => {
  unlisten?.()
  if (blobUrl.value) URL.revokeObjectURL(blobUrl.value)
})

watch(() => props.artist, async () => {
  if (blobUrl.value) URL.revokeObjectURL(blobUrl.value)
  blobUrl.value = null
  await load()
})

watch(() => appStore.useOnlineArtistArtwork, async () => {
  if (blobUrl.value) URL.revokeObjectURL(blobUrl.value)
  blobUrl.value = null
  await load()
})
</script>

<template>
  <div class="w-full h-full flex items-center justify-center overflow-hidden bg-foreground/5">
    <img v-if="blobUrl" :src="blobUrl" :alt="artist" class="w-full h-full object-cover" />
    <User v-else class="w-1/3 h-1/3 text-foreground opacity-40" />
  </div>
</template>
