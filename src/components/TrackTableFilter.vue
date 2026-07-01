<script setup lang="ts">
import { ListFilter } from 'lucide-vue-next'
import FilterDropdown from './FilterDropdown.vue'

defineProps<{
  simpleMode?: boolean
}>()
</script>

<template>
  <template v-if="!simpleMode">
    <FilterDropdown>
      <template #trigger="{ open }">
        <button :class="['p-2 rounded-xl border transition-all', open ? 'bg-foreground/[0.08] border-foreground/20' : 'bg-foreground/[0.04] border-foreground/10 hover:bg-foreground/[0.06]']">
          <ListFilter class="w-3.5 h-3.5" />
        </button>
      </template>
      <div class="p-2 text-xs text-foreground/60 text-center">Column settings</div>
    </FilterDropdown>
  </template>
</template>
