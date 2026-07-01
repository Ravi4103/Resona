<script setup lang="ts">
import { LayoutGrid, List, ArrowUp, ArrowDown, ListFilter } from 'lucide-vue-next'
import FilterDropdown from './FilterDropdown.vue'

type SortCol = 'title' | 'artist' | 'year'
type SortDir = 'asc' | 'desc'

const props = defineProps<{
  viewMode: 'grid' | 'list'
  sortColumn: SortCol | null
  sortDir: SortDir
}>()

const emit = defineEmits<{
  'update:viewMode': [mode: 'grid' | 'list']
  'update:sortColumn': [col: SortCol]
  'update:sortDir': [dir: SortDir]
}>()

function selectSortCol(col: SortCol) {
  if (props.sortColumn === col) {
    emit('update:sortDir', props.sortDir === 'asc' ? 'desc' : 'asc')
  } else {
    emit('update:sortColumn', col)
    emit('update:sortDir', 'asc')
  }
}

const SORT_LABELS: Record<SortCol, string> = {
  title: 'Title',
  artist: 'Artist',
  year: 'Year',
}
</script>

<template>
  <FilterDropdown :panel-offset-y="4">
    <template #trigger="{ open }">
      <button
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg ring-1 transition-colors text-sm font-medium"
        :class="open
          ? 'bg-foreground/10 ring-foreground/20 text-foreground'
          : 'ring-foreground/[0.1] text-foreground opacity-70 hover:text-foreground hover:bg-foreground/[0.04]'"
      >
        <ListFilter class="w-3.5 h-3.5" />
        <span>Filter</span>
      </button>
    </template>

    <!-- Layout section -->
    <p class="text-[10px] font-semibold text-foreground opacity-60 uppercase tracking-widest px-1 mb-2">
      Layout
    </p>
    <div class="flex flex-col gap-0.5">
      <div
        class="flex items-center gap-2.5 px-1.5 py-1.5 rounded-lg hover:bg-foreground/[0.06] cursor-pointer transition-colors"
        :class="{ 'bg-foreground/[0.08]': viewMode === 'grid' }"
        @click="emit('update:viewMode', 'grid')"
      >
        <LayoutGrid class="w-4 h-4 text-foreground/70" />
        <span class="text-sm text-foreground opacity-90">Grid View</span>
        <div v-if="viewMode === 'grid'" class="ml-auto w-1.5 h-1.5 rounded-full bg-primary" />
      </div>
      <div
        class="flex items-center gap-2.5 px-1.5 py-1.5 rounded-lg hover:bg-foreground/[0.06] cursor-pointer transition-colors"
        :class="{ 'bg-foreground/[0.08]': viewMode === 'list' }"
        @click="emit('update:viewMode', 'list')"
      >
        <List class="w-4 h-4 text-foreground/70" />
        <span class="text-sm text-foreground opacity-90">List View</span>
        <div v-if="viewMode === 'list'" class="ml-auto w-1.5 h-1.5 rounded-full bg-primary" />
      </div>
    </div>

    <!-- Sort section -->
    <div class="mt-2 pt-2 border-t border-foreground/10">
      <p class="text-[10px] font-semibold text-foreground opacity-60 uppercase tracking-widest px-1 mb-2">
        Sort By
      </p>
      <div class="flex flex-col gap-0.5">
        <div
          v-for="col in (['title', 'artist', 'year'] as const)"
          :key="col"
          class="flex items-center gap-2.5 px-1.5 py-1.5 rounded-lg hover:bg-foreground/[0.06] cursor-pointer transition-colors"
          :class="{ 'bg-foreground/[0.08]': sortColumn === col }"
          @click="selectSortCol(col)"
        >
          <ArrowUp v-if="sortColumn === col && sortDir === 'asc'" class="w-4 h-4 text-primary" />
          <ArrowDown v-else-if="sortColumn === col && sortDir === 'desc'" class="w-4 h-4 text-primary" />
          <span v-else class="w-4 h-4 inline-block" />
          <span
            class="text-sm text-foreground opacity-90"
            :class="{ 'text-primary opacity-100': sortColumn === col }"
          >
            {{ SORT_LABELS[col] }}
          </span>
          <div v-if="sortColumn === col" class="ml-auto w-1.5 h-1.5 rounded-full bg-primary" />
        </div>
      </div>
    </div>
  </FilterDropdown>
</template>
