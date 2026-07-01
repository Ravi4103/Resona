<script setup lang="ts">
import { User } from 'lucide-vue-next'

defineProps<{
  artist: { name: string; trackCount: number }
  variant?: 'card' | 'avatar'
}>()

const emit = defineEmits<{
  click: [name: string]
}>()
</script>

<template>
  <div v-if="variant === 'avatar'"
    class="w-full h-full flex items-center justify-center"
    @click="emit('click', artist.name)">
    <div class="w-full h-full flex items-center justify-center text-foreground opacity-40 group-hover:bg-foreground/10 transition-colors">
      <User class="w-1/2 h-1/2" />
    </div>
  </div>

  <div v-else class="group cursor-pointer text-center" @click="emit('click', artist.name)">
    <div class="aspect-square bg-foreground/5 rounded-full ring-1 ring-foreground/[0.06] overflow-hidden relative mb-3 transition-all flex items-center justify-center">
      <div class="w-full h-full flex items-center justify-center text-foreground opacity-40 group-hover:bg-foreground/10 transition-colors">
        <User class="w-1/2 h-1/2" />
      </div>
    </div>
    <div class="space-y-1 px-1">
      <h3 class="font-medium text-sm truncate group-hover:text-foreground transition-colors">{{ artist.name || 'Unknown Artist' }}</h3>
      <p class="text-xs text-foreground opacity-60">{{ artist.trackCount }} {{ artist.trackCount === 1 ? 'track' : 'tracks' }}</p>
    </div>
  </div>
</template>
