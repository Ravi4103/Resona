<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowLeft, Play, Shuffle, MoreVertical } from 'lucide-vue-next'

const props = withDefaults(defineProps<{
  title: string
  artist: string
  artworkSrc?: string | null
  dominantColor?: string | null
  trackCount?: number
}>(), { artworkSrc: null, dominantColor: null, trackCount: 0 })

const emit = defineEmits<{
  play: []
  shuffle: []
  contextmenu: [e: MouseEvent]
  artistClick: [name: string]
}>()

const router = useRouter()

const dynamicSurface = computed(() => {
  if (props.dominantColor) return props.dominantColor
  return 'var(--bg-glass)'
})
</script>

<template>
  <div class="bg-gradient-to-b from-dynamic-surface to-transparent"
    :style="{ '--dynamic-surface': dynamicSurface }"
    @contextmenu.prevent="(e) => emit('contextmenu', e)">
    <div class="pt-4 px-4 md:pt-5 md:px-8 flex items-center justify-between">
      <button @click="router.back()" class="p-2 relative z-[99] hover:bg-foreground/[0.06] rounded-full transition-colors">
        <ArrowLeft class="w-6 h-6" />
      </button>
      <slot name="top-right" />
    </div>

    <div class="px-6 md:px-10 pb-8 md:pb-10 pt-4 flex flex-col md:flex-row gap-8 items-end">
      <div class="w-48 h-48 md:w-56 md:h-56 lg:w-64 lg:h-64 rounded-lg shadow-2xl overflow-hidden ring-1 ring-foreground/[0.08] bg-foreground/5 flex-shrink-0">
        <img v-if="artworkSrc" :src="artworkSrc" :alt="title" class="w-full h-full object-cover" />
        <div v-else class="w-full h-full flex items-center justify-center text-foreground opacity-30">
          <svg class="w-24 h-24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="12" cy="12" r="10" />
            <circle cx="12" cy="12" r="3" />
          </svg>
        </div>
      </div>

      <div class="flex-1 space-y-4 min-w-0">
        <div class="space-y-2">
          <h1 class="text-2xl sm:text-3xl md:text-4xl lg:text-5xl font-bold tracking-tight line-clamp-2 leading-snug">
            {{ title }}
          </h1>
          <div class="flex items-center gap-2 text-foreground font-semibold min-w-0">
            <span class="truncate cursor-pointer hover:text-primary transition-colors" @click="emit('artistClick', artist)">
              {{ artist }}
            </span>
          </div>
          <div class="flex flex-wrap items-center gap-x-4 gap-y-2 text-foreground opacity-60">
            <slot name="metadata" />
          </div>
        </div>

        <div class="flex items-center gap-4 pt-2">
          <button @click="emit('play')"
            class="flex items-center gap-2 px-6 py-2.5 bg-primary text-primary-foreground rounded-full font-medium hover:scale-105 transition-transform shadow-lg shadow-primary/20">
            <Play class="w-5 h-5 fill-current" /> Play
          </button>
          <div class="flex gap-2">
            <button @click="emit('shuffle')"
              class="flex items-center gap-2 px-5 py-2.5 bg-foreground/10 text-foreground rounded-full font-medium hover:bg-foreground/20 transition-all">
              <Shuffle class="w-4 h-4" />
            </button>
            <button @click="(e) => emit('contextmenu', e)"
              class="flex items-center gap-2 px-3 py-2.5 bg-foreground/10 text-foreground rounded-full font-medium hover:bg-foreground/20 transition-all">
              <MoreVertical class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
