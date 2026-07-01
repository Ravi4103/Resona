<script setup lang="ts">
import { ref, computed } from 'vue'
import { Search, Play } from 'lucide-vue-next'
import { RecycleScroller } from 'vue-virtual-scroller'

const props = defineProps<{
  title: string
  items: { id: string; name: string; trackCount?: number }[]
  isLoading?: boolean
  selectedId?: string
  searchPlaceholder?: string
  icon?: any
}>()

const emit = defineEmits<{
  select: [id: string]
  play: [item: { id: string; name: string }]
}>()

const searchQuery = ref('')

function foldUnicode(s: string): string {
  return s.toLowerCase().normalize('NFD').replace(/[\u0300-\u036f]/g, '')
}

const filteredItems = computed(() => {
  if (!searchQuery.value) return props.items
  const q = foldUnicode(searchQuery.value)
  return props.items.filter(item => foldUnicode(item.name || '').includes(q))
})

function clearSearch() { searchQuery.value = '' }
</script>

<template>
  <div class="h-full flex overflow-hidden bg-background">
    <div class="w-[280px] border-r border-foreground/[0.06] flex flex-col overflow-hidden bg-background select-none">
      <div class="p-6 pb-3">
        <div class="flex items-center justify-between mb-4">
          <h1 class="text-2xl font-bold">{{ title }}</h1>
          <span v-if="!isLoading && items.length > 0"
            class="text-xs text-foreground opacity-40 tabular-nums bg-foreground/5 px-2 py-0.5 rounded-full">
            {{ filteredItems.length }}{{ searchQuery ? `/${items.length}` : '' }}
          </span>
        </div>
        <div class="relative">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-foreground opacity-40 pointer-events-none" />
          <input v-model="searchQuery" type="text" :placeholder="searchPlaceholder || 'Search...'"
            class="w-full pl-9 pr-8 py-2 text-sm rounded-lg bg-foreground/[0.05] ring-1 ring-foreground/10 text-foreground placeholder:text-foreground/40 outline-none focus:ring-primary/40 transition-all" />
          <button v-if="searchQuery" @click="clearSearch"
            class="absolute right-2.5 top-1/2 -translate-y-1/2 text-foreground opacity-40 hover:opacity-80 transition-opacity">
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <div class="flex-1 overflow-hidden">
        <div v-if="isLoading" class="h-full flex items-center justify-center">
          <div class="animate-spin rounded-full h-8 w-8 border-2 border-primary border-t-transparent" />
        </div>
        <div v-else-if="filteredItems.length === 0"
          class="h-full flex flex-col items-center justify-center gap-3 text-foreground opacity-40 px-6">
          <component :is="icon" v-if="icon" class="w-10 h-10 opacity-30" />
          <p class="text-sm text-center">{{ searchQuery ? `No results for "${searchQuery}"` : `No ${title.toLowerCase()} found` }}</p>
        </div>
        <RecycleScroller v-else :items="filteredItems" :item-size="52" key-field="id" class="h-full px-3 pb-3"
          v-slot="{ item }">
          <div @click="emit('select', item.id)"
            :class="[
              'flex items-center gap-3 p-2 rounded-lg group transition-all cursor-pointer mb-1',
              selectedId === item.id ? 'bg-primary/10 text-foreground ring-1 ring-primary/20' : 'hover:bg-foreground/[0.04]'
            ]">
            <div :class="[
              'w-8 h-8 rounded-full flex items-center justify-center flex-shrink-0 ring-1 transition-colors overflow-hidden',
              selectedId === item.id ? 'bg-primary/15 ring-primary/30' : 'bg-foreground/5 ring-foreground/[0.06] group-hover:ring-foreground/[0.12]'
            ]">
              <slot name="item-icon" :item="item">
                <component :is="icon" v-if="icon" class="w-4 h-4" :class="selectedId === item.id ? 'text-primary' : ''" />
                <span v-else class="text-xs font-bold">{{ item.name.charAt(0).toUpperCase() }}</span>
              </slot>
            </div>
            <div class="flex-1 min-w-0">
              <div class="truncate font-medium text-sm" :class="selectedId === item.id ? 'text-primary' : ''">
                {{ item.name || 'Unknown' }}
              </div>
              <div v-if="item.trackCount !== undefined" class="text-xs text-foreground opacity-50 mt-0.5">
                {{ item.trackCount }} track{{ item.trackCount === 1 ? '' : 's' }}
              </div>
            </div>
            <button @click.stop="emit('play', item)"
              class="p-1.5 opacity-0 group-hover:opacity-100 bg-primary text-primary-foreground rounded-full shadow-lg transition-all scale-75 group-hover:scale-100 hover:scale-110">
              <Play class="w-3 h-3 fill-current" />
            </button>
          </div>
        </RecycleScroller>
      </div>
    </div>

    <div class="flex-1 overflow-hidden bg-background relative">
      <slot v-if="selectedId" />
      <div v-if="!selectedId && !isLoading"
        class="h-full flex flex-col items-center justify-center text-foreground opacity-40 gap-4">
        <component :is="icon" v-if="icon" class="w-20 h-20 opacity-10" />
        <div class="text-center">
          <p class="text-base font-medium">Select an item</p>
          <p class="text-sm opacity-60 mt-1">Choose from the list on the left</p>
        </div>
      </div>
    </div>
  </div>
</template>
