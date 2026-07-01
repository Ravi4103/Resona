<script setup lang="ts">
import { RouterLink } from 'vue-router'
import type { Component } from 'vue'
import { usePlayerStore } from '../stores/player'
import { computed } from 'vue'

const props = defineProps<{
  to: string
  icon: Component
  label: string
}>()

defineEmits<{
  (e: 'contextmenu', event: MouseEvent): void
  (e: 'dblclick', event: MouseEvent): void
}>()

const playerStore = usePlayerStore()

// Show a pulsing dot on Home if music is playing
const showPlayingDot = computed(() =>
  props.to === '/' && playerStore.isPlaying
)
</script>

<template>
  <div class="relative group">
    <RouterLink :to="to"
      class="flex items-center gap-3 px-3 py-2 rounded-lg transition-all text-foreground opacity-70 hover:text-foreground hover:bg-foreground/[0.05] hover:opacity-100"
      :class="{ 'pr-8': $slots.actions }"
      active-class="bg-foreground/[0.08] !text-primary !opacity-100 font-semibold"
      @contextmenu="$emit('contextmenu', $event)"
      @dblclick="$emit('dblclick', $event)">
      <component :is="icon" class="w-4 h-4 flex-shrink-0" />
      <span class="text-sm truncate flex-1">{{ label }}</span>
      <!-- Playing indicator dot -->
      <span v-if="showPlayingDot"
        class="w-1.5 h-1.5 rounded-full bg-primary animate-pulse flex-shrink-0" />
    </RouterLink>

    <div v-if="$slots.actions" class="absolute right-2 top-1/2 -translate-y-1/2 flex items-center">
      <slot name="actions" />
    </div>
  </div>
</template>
