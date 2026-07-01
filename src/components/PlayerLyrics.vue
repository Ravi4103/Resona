<script setup lang="ts">
import { computed } from 'vue'
import PlainLyricsView from './PlainLyricsView.vue'
import SyncedLyricsView from './SyncedLyricsView.vue'
import { useLyrics } from '../composables/useLyrics'

const props = defineProps<{
  lyrics?: string
  isLoading?: boolean
  currentPosition?: number
}>()

const emit = defineEmits<{
  seek: [time: number]
}>()

const lyricsRef = computed(() => props.lyrics)
const { isSynced, syncedLines, plainLines } = useLyrics(lyricsRef)

const syncedLinesMapped = computed(() =>
  syncedLines.value.map(l => ({
    time: l.time,
    text: l.text,
    secondary: (l as any).secondary,
  }))
)
</script>

<template>
  <div class="h-full w-full flex flex-col overflow-hidden">
    <div v-if="isLoading" class="flex-1 overflow-y-auto px-8 py-48">
      <div class="max-w-2xl mx-auto space-y-10">
        <div
          v-for="(width, i) in ['w-3/4', 'w-1/2', 'w-5/6', 'w-2/3', 'w-1/3', 'w-4/5', 'w-1/2', 'w-2/3', 'w-3/4', 'w-1/4']"
          :key="i"
          class="h-8 md:h-12 rounded-lg bg-white/[0.06] animate-pulse"
          :class="width"
        />
      </div>
    </div>

    <div
      v-else-if="!lyrics"
      class="flex-1 flex flex-col items-center justify-center text-white/20 p-12 text-center"
    >
      <p class="text-lg font-medium text-white/40">Lyrics not available</p>
      <p class="text-sm text-white/20 mt-2">Try searching for lyrics online</p>
    </div>

    <SyncedLyricsView
      v-else-if="isSynced"
      class="flex-1"
      :lines="syncedLinesMapped"
      :current-position="currentPosition ?? 0"
      @seek="(time) => emit('seek', time)"
    />

    <PlainLyricsView
      v-else
      class="flex-1"
      :lines="plainLines"
    />
  </div>
</template>
