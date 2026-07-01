<script setup lang="ts">
import { ref } from 'vue'
import { ArrowDown, Loader2 } from 'lucide-vue-next'

const emit = defineEmits<{
  refresh: []
}>()

const THRESHOLD = 80
const state = ref<'idle' | 'pulling' | 'releasing' | 'refreshing'>('idle')
const pullDistance = ref(0)
const startY = ref(0)
const containerRef = ref<HTMLElement | null>(null)

function onPointerDown(e: PointerEvent) {
  if (state.value === 'refreshing') return
  const scrollContainer = containerRef.value?.querySelector('.scroll-container')
  if (scrollContainer && scrollContainer.scrollTop > 0) return
  startY.value = e.clientY
  state.value = 'pulling'
  document.addEventListener('pointermove', onPointerMove)
  document.addEventListener('pointerup', onPointerUp)
  document.addEventListener('pointercancel', onPointerUp)
}

function onPointerMove(e: PointerEvent) {
  if (state.value !== 'pulling' && state.value !== 'releasing') return
  const delta = e.clientY - startY.value
  if (delta <= 0) {
    pullDistance.value = 0
    state.value = 'idle'
    cleanup()
    return
  }
  pullDistance.value = Math.min(delta * 0.5, 160)
  if (pullDistance.value >= THRESHOLD) {
    state.value = 'releasing'
  } else {
    state.value = 'pulling'
  }
}

function onPointerUp() {
  cleanup()
  if (pullDistance.value >= THRESHOLD) {
    state.value = 'refreshing'
    emit('refresh')
  } else {
    state.value = 'idle'
  }
  pullDistance.value = 0
}

function cleanup() {
  document.removeEventListener('pointermove', onPointerMove)
  document.removeEventListener('pointerup', onPointerUp)
  document.removeEventListener('pointercancel', onPointerUp)
}

function doneRefreshing() {
  state.value = 'idle'
}

defineExpose({ doneRefreshing })
</script>

<template>
  <div ref="containerRef" class="relative h-full" @pointerdown="onPointerDown">
    <!-- Pull indicator -->
    <div class="absolute left-0 right-0 flex justify-center pointer-events-none z-10 transition-opacity"
      :class="pullDistance > 0 ? 'opacity-100' : 'opacity-0'"
      :style="{ top: `${Math.max(0, pullDistance - 40)}px` }">
      <div class="w-8 h-8 rounded-full bg-foreground/10 flex items-center justify-center backdrop-blur-sm">
        <Loader2 v-if="state === 'refreshing'" class="w-4 h-4 animate-spin text-primary" />
        <ArrowDown v-else class="w-4 h-4 text-foreground/60 transition-transform duration-200"
          :class="state === 'releasing' ? 'rotate-180' : ''" />
      </div>
    </div>

    <!-- Content wrapper with translate -->
    <div class="h-full overflow-hidden">
      <div class="scroll-container h-full overflow-y-auto custom-scrollbar"
        :style="{ transform: pullDistance > 0 ? `translateY(${pullDistance}px)` : '', transition: state === 'idle' || state === 'refreshing' ? 'transform 0.3s ease' : '' }">
        <slot />
      </div>
    </div>
  </div>
</template>
