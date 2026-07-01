<script setup lang="ts">
import { computed } from 'vue'
import { Calendar, Tag, Music, Clock } from 'lucide-vue-next'
import type { Track } from '../lib/invoke'

const props = defineProps<{ tracks: Track[] }>()

const year = computed(() => props.tracks[0]?.year || 0)
const genre = computed(() => props.tracks[0]?.raw_genre_names || props.tracks[0]?.genre || '')
const trackCount = computed(() => props.tracks.length)
const totalMinutes = computed(() => Math.floor(props.tracks.reduce((a, t) => a + t.duration, 0) / 60))
</script>

<template>
  <div class="flex flex-wrap items-center gap-4 text-sm">
    <div v-if="year" class="flex items-center gap-1.5">
      <Calendar class="w-3.5 h-3.5" />
      <span>{{ year }}</span>
    </div>
    <div v-if="genre" class="flex items-center gap-1.5">
      <Tag class="w-3.5 h-3.5" />
      <span>{{ genre }}</span>
    </div>
    <div class="flex items-center gap-1.5">
      <Music class="w-3.5 h-3.5" />
      <span>{{ trackCount }} track{{ trackCount === 1 ? '' : 's' }}</span>
    </div>
    <div class="flex items-center gap-1.5">
      <Clock class="w-3.5 h-3.5" />
      <span>{{ totalMinutes }} min</span>
    </div>
  </div>
</template>
