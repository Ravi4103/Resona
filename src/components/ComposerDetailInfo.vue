<script setup lang="ts">
import { computed } from 'vue'
import { Disc, Music } from 'lucide-vue-next'
import type { Track } from '../lib/invoke'

const props = defineProps<{ tracks: Track[] }>()

const albumCount = computed(() => {
  const albums = new Set(props.tracks.map(t => t.album).filter(Boolean))
  return albums.size
})

const trackCount = computed(() => props.tracks.length)
</script>

<template>
  <div class="flex flex-wrap items-center gap-4 text-sm">
    <span class="flex items-center gap-1.5">
      <Disc class="w-3.5 h-3.5" />
      {{ albumCount }} album{{ albumCount === 1 ? '' : 's' }}
    </span>
    <span class="flex items-center gap-1.5">
      <Music class="w-3.5 h-3.5" />
      {{ trackCount }} track{{ trackCount === 1 ? '' : 's' }}
    </span>
  </div>
</template>
