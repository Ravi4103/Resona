<script setup lang="ts">
import { computed, ref, watch, onUnmounted } from 'vue'

const props = withDefaults(defineProps<{
  modelValue: number
  min?: number
  max?: number
  step?: number
}>(), { min: 0, max: 100, step: 0.01 })

const emit = defineEmits<{
  'update:modelValue': [value: number]
  'mousedown': [e: MouseEvent]
  'mouseup': [e: MouseEvent]
  'touchstart': [e: TouchEvent]
  'touchend': [e: TouchEvent]
}>()

const isInteracting = ref(false)
const localValue = ref(props.modelValue)

const gracePeriodActive = ref(false)
let graceTimeout: ReturnType<typeof setTimeout> | null = null

watch(() => props.modelValue, (newVal) => {
  if (!isInteracting.value && !gracePeriodActive.value) {
    localValue.value = newVal
  }
})

const fillPct = computed(() => {
  const range = props.max - props.min
  if (range === 0) return 0
  return Math.min(100, Math.max(0, ((localValue.value - props.min) / range) * 100))
})

function handleInput(e: Event) {
  const val = Number((e.target as HTMLInputElement).value)
  localValue.value = val
  emit('update:modelValue', val)
}

function handleStart(e: MouseEvent | TouchEvent) {
  isInteracting.value = true
  if (graceTimeout) { clearTimeout(graceTimeout); graceTimeout = null }
  gracePeriodActive.value = false
  if (e instanceof MouseEvent) emit('mousedown', e)
  else emit('touchstart', e)
}

function handleEnd(e: MouseEvent | TouchEvent) {
  isInteracting.value = false
  gracePeriodActive.value = true
  if (graceTimeout) clearTimeout(graceTimeout)
  graceTimeout = setTimeout(() => {
    gracePeriodActive.value = false
    localValue.value = props.modelValue
  }, 1000)
  if (e instanceof MouseEvent) emit('mouseup', e)
  else emit('touchend', e)
}

onUnmounted(() => {
  if (graceTimeout) clearTimeout(graceTimeout)
})
</script>

<template>
  <div class="relative h-4 flex items-center group/slider cursor-pointer select-none">
    <div class="absolute w-full h-1 rounded-full bg-foreground/15 pointer-events-none">
      <div class="h-full rounded-full bg-foreground"
        :style="{ width: `${fillPct}%` }" />
    </div>
    <input type="range" :min="min" :max="max" :step="step" :value="localValue"
      class="custom-slider absolute inset-0 w-full bg-transparent appearance-none cursor-pointer z-10"
      @input="handleInput" @mousedown="handleStart" @mouseup="handleEnd"
      @touchstart="handleStart" @touchend="handleEnd" />
  </div>
</template>

<style scoped>
.custom-slider::-webkit-slider-thumb {
  appearance: none;
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  background: white;
  border-radius: 50%;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  opacity: 0;
  transition: opacity 150ms ease;
}
.group\/slider:hover .custom-slider::-webkit-slider-thumb {
  opacity: 1;
}
.custom-slider::-moz-range-thumb {
  width: 12px;
  height: 12px;
  background: white;
  border-radius: 50%;
  border: none;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  opacity: 0;
  transition: opacity 150ms ease;
}
.group\/slider:hover .custom-slider::-moz-range-thumb {
  opacity: 1;
}
.custom-slider::-webkit-slider-runnable-track {
  background: transparent;
}
.custom-slider::-moz-range-track {
  background: transparent;
}
</style>
