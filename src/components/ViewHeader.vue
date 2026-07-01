<script setup lang="ts">
import { Search } from 'lucide-vue-next'

defineProps<{
  title: string
  icon?: any
  subtitle?: string
  searchPlaceholder?: string
}>()

const searchQuery = defineModel<string>({ default: '' })
</script>

<template>
  <div class="px-6 py-5 border-b border-foreground/[0.06] select-none bg-background/60 backdrop-blur-sm sticky top-0 z-10">
    <div class="flex items-center justify-between gap-4">
      <div class="flex items-center gap-3 min-w-0">
        <div v-if="icon" class="p-2 bg-primary/10 rounded-xl flex-shrink-0">
          <component :is="icon" class="w-5 h-5 text-primary" />
        </div>
        <div class="min-w-0">
          <h1 class="text-2xl font-bold truncate">{{ title }}</h1>
          <p v-if="subtitle" class="text-xs text-foreground opacity-50 mt-0.5">{{ subtitle }}</p>
        </div>
      </div>
      <div class="flex items-center gap-2 flex-shrink-0">
        <slot name="actions" />
        <div v-if="searchPlaceholder" class="relative w-52">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-foreground opacity-40 pointer-events-none" />
          <input v-model="searchQuery" type="text" :placeholder="searchPlaceholder"
            class="w-full pl-9 pr-4 py-2 text-sm rounded-lg bg-foreground/[0.05] ring-1 ring-foreground/10 text-foreground placeholder:text-foreground/40 outline-none focus:ring-primary/40 transition-all" />
        </div>
      </div>
    </div>
  </div>
</template>
