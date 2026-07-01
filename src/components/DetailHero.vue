<script setup lang="ts">
import { useRouter } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  title?: string
  dominantColor?: string | null
}>(), { dominantColor: null })

const router = useRouter()

const dynamicSurface = computed(() => {
  if (props.dominantColor) return props.dominantColor
  return 'var(--bg-glass)'
})
</script>

<template>
  <div class="flex flex-col bg-gradient-to-b from-dynamic-surface to-transparent"
    :style="{ '--dynamic-surface': dynamicSurface }">
    <div class="pt-4 px-4 md:pt-5 md:px-8 flex items-center justify-between">
      <button @click="router.back()" class="p-2 relative z-[99] hover:bg-foreground/[0.06] rounded-full transition-colors">
        <ArrowLeft class="w-6 h-6" />
      </button>
      <slot name="top-right" />
    </div>

    <div class="px-8 pb-8 md:px-12 md:pb-12 pt-4 flex flex-col md:flex-row gap-8 items-end">
      <slot name="artwork" />
      <div class="flex-1 space-y-4 min-w-0">
        <div class="space-y-2">
          <slot name="title">
            <h1 v-if="title" class="text-2xl sm:text-3xl md:text-4xl lg:text-5xl font-bold tracking-tight line-clamp-2 leading-snug">
              {{ title }}
            </h1>
          </slot>
          <div class="flex flex-wrap items-center gap-x-4 gap-y-2 text-foreground opacity-60">
            <slot name="metadata" />
          </div>
        </div>
        <div class="flex items-center gap-4 pt-2">
          <slot name="actions" />
        </div>
      </div>
    </div>
  </div>
</template>
