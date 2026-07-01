<script setup lang="ts">
import { ref, computed } from 'vue'
import { useAppStore, colorDefaults, type ColorKey } from '../stores/app'

const props = withDefaults(defineProps<{
  label: string
  colorKey: ColorKey
  presets?: string[]
}>(), {
  presets: () => [
    '#e11d48', '#3b82f6', '#8b5cf6', '#10b981',
    '#f97316', '#0d9488', '#db2777', '#4f46e5',
    '#d97706', '#0284c7',
  ],
})

const appStore = useAppStore()
const showPicker = ref(false)

const currentHex = computed(() => appStore.colors[props.colorKey])

const isDefault = computed(() => currentHex.value === colorDefaults[props.colorKey])

function handleInput(e: Event) {
  const val = (e.target as HTMLInputElement).value
  appStore.applyColor(props.colorKey, val)
}

function selectPreset(hex: string) {
  appStore.applyColor(props.colorKey, hex)
}

function reset() {
  appStore.applyColor(props.colorKey, colorDefaults[props.colorKey])
}
</script>

<template>
  <div class="flex items-center justify-between gap-x-3">
    <span class="text-sm font-medium shrink-0">{{ label }}</span>
    <div class="flex items-center gap-2">
      <div v-if="!isDefault" @click="reset" class="w-4 h-4 rounded-full border border-foreground/20 flex items-center justify-center cursor-pointer hover:bg-foreground/10 transition-colors" title="Reset to default">
        <svg class="w-2.5 h-2.5 text-foreground/60" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 4v6h6"/><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/></svg>
      </div>
      <template v-if="currentHex">
        <div class="relative">
          <div
            class="w-6 h-6 rounded-full border-2 border-foreground/10 cursor-pointer transition-transform hover:scale-110"
            :style="{ backgroundColor: currentHex }"
            @click="showPicker = !showPicker"
          />
          <input
            v-if="showPicker"
            :value="currentHex"
            @input="handleInput"
            type="color"
            class="absolute top-8 right-0 z-50 w-8 h-8 p-0 border-0 cursor-pointer"
            @blur="showPicker = false"
          />
        </div>
        <input
          :value="currentHex"
          @input="handleInput"
          class="w-20 bg-foreground/[0.04] border border-foreground/10 rounded-lg px-2 py-1 text-xs font-mono text-foreground/80 focus:outline-none focus:border-primary/50 transition-colors"
          placeholder="#hex"
          maxlength="7"
        />
      </template>
      <span v-else class="text-xs text-foreground/40 italic mr-1">Auto (from artwork)</span>
      <div class="flex gap-1">
        <button
          v-for="hex in presets"
          :key="hex"
          @click="selectPreset(hex)"
          class="w-4 h-4 rounded-full border border-foreground/10 transition-transform hover:scale-125"
          :class="{ 'ring-1 ring-primary/50 scale-110': currentHex === hex }"
          :style="{ backgroundColor: hex }"
        />
      </div>
    </div>
  </div>
</template>
