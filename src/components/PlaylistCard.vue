<script setup lang="ts">
import { ListMusic } from 'lucide-vue-next'
import type { Track } from '../lib/invoke'

defineProps<{
  playlist: { id: string; name: string }
  tracks?: Track[]
}>()

const emit = defineEmits<{
  click: [id: string]
}>()
</script>

<template>
  <div class="group cursor-pointer" @click="emit('click', playlist.id)">
    <div class="aspect-square bg-foreground/5 rounded-lg ring-1 ring-foreground/[0.06] overflow-hidden relative mb-3 transition-all flex items-center justify-center">
      <div class="w-full h-full flex items-center justify-center text-foreground opacity-40 group-hover:bg-foreground/10 transition-colors">
        <ListMusic class="w-1/3 h-1/3" />
      </div>
    </div>
    <div class="space-y-1 px-1">
      <h3 class="font-medium text-sm truncate group-hover:text-foreground transition-colors">{{ playlist.name || 'Unknown' }}</h3>
      <p class="text-xs text-foreground opacity-60">Playlist</p>
    </div>
  </div>
</template>
