<script setup lang="ts">
import { ref, computed } from 'vue'

const props = defineProps<{
  progressPercent: number
  position: number
  duration: number
}>()

const emit = defineEmits<{
  seek: [value: number]
}>()

const isSeeking = ref(false)
const seekValue = ref(0)

const displayPosition = computed(() =>
  isSeeking.value ? (seekValue.value / 100) * props.duration : props.position,
)

function formatTime(secs: number): string {
  if (!secs || !isFinite(secs)) return '0:00'
  const m = Math.floor(secs / 60)
  const s = Math.floor(secs % 60)
  return `${m}:${s.toString().padStart(2, '0')}`
}

function onSeekStart() {
  isSeeking.value = true
  seekValue.value = props.progressPercent
}

function onSeekEnd() {
  emit('seek', (seekValue.value / 100) * props.duration)
  isSeeking.value = false
}
</script>

<template>
  <div class="w-full max-w-sm space-y-1.5">
    <input
      type="range"
      :value="isSeeking ? seekValue : progressPercent"
      min="0"
      max="100"
      step="0.1"
      class="w-full accent-primary"
      @input="(v) => (seekValue = parseFloat((v.target as HTMLInputElement).value))"
      @mousedown="onSeekStart"
      @mouseup="onSeekEnd"
      @touchstart="onSeekStart"
      @touchend="onSeekEnd"
    />
    <div class="flex justify-between text-[10.5px] text-white/60 tabular-nums">
      <span>{{ formatTime(displayPosition) }}</span>
      <span>{{ formatTime(duration) }}</span>
    </div>
  </div>
</template>
