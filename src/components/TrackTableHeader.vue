<script setup lang="ts">
import { ref } from 'vue'
import SortableHeaderCell from './SortableHeaderCell.vue'
import type { ColumnDef, ColumnKey } from '../composables/useTrackTableSettings'

withDefaults(defineProps<{
  gridTemplateColumns: string
  orderedVisibleColumns: ColumnDef[]
  sortColumn: string | null
  sortDir: 'asc' | 'desc'
  variant?: 'default' | 'glass'
}>(), {
  variant: 'default',
})

const emit = defineEmits<{
  sort: [col: string]
  reorderColumn: [from: ColumnKey, to: ColumnKey]
  resizeColumn: [key: ColumnKey, width: number]
}>()

const resizing = ref<ColumnKey | null>(null)
const dragFrom = ref<ColumnKey | null>(null)

function handleMouseDown(e: MouseEvent, col: ColumnDef) {
  if (!col.draggable) return
  e.preventDefault()
  const startX = e.clientX
  const startWidth = (e.target as HTMLElement).parentElement?.getBoundingClientRect().width ?? 100
  resizing.value = col.key

  const onMove = (ev: MouseEvent) => {
    const newWidth = Math.max(col.minWidthPx, startWidth + (ev.clientX - startX))
    emit('resizeColumn', col.key, newWidth)
  }
  const onUp = () => {
    resizing.value = null
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
  }
  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}

function handleDragStart(e: DragEvent, col: ColumnDef) {
  if (!col.draggable) return
  dragFrom.value = col.key
  e.dataTransfer?.setData('text/plain', col.key)
}

function handleDragOver(e: DragEvent, col: ColumnDef) {
  if (!col.draggable || !dragFrom.value) return
  e.preventDefault()
}

function handleDrop(e: DragEvent, col: ColumnDef) {
  e.preventDefault()
  if (dragFrom.value && dragFrom.value !== col.key) {
    emit('reorderColumn', dragFrom.value, col.key)
  }
  dragFrom.value = null
}

function handleDragEnd() {
  dragFrom.value = null
}
</script>

<template>
  <div class="grid items-center text-xs font-medium text-foreground/40 border-b border-foreground/[0.06] select-none sticky top-0 z-10"
    :class="variant === 'glass' ? 'bg-transparent' : 'bg-background/80 backdrop-blur-sm'"
    :style="{ gridTemplateColumns }">
    <template v-for="col in orderedVisibleColumns" :key="col.key">
      <div v-if="col.key === 'dnd'" class="flex items-center justify-center px-2">
        <span class="w-4 h-4" />
      </div>

      <div v-else-if="col.key === 'index'" class="flex items-center justify-center px-2">#</div>

      <div v-else-if="col.key === 'favorite' || col.key === 'context_menu'" class="flex items-center justify-center px-2">
        <span class="w-4 h-4" />
      </div>

      <div v-else-if="col.sortable && col.label"
        class="relative flex items-center min-w-0"
        :draggable="col.draggable"
        @dragstart="handleDragStart($event, col)"
        @dragover="handleDragOver($event, col)"
        @drop="handleDrop($event, col)"
        @dragend="handleDragEnd">
        <SortableHeaderCell :active="sortColumn === col.key" :dir="sortColumn === col.key ? sortDir : 'asc'"
          @click="emit('sort', col.key)">
          {{ col.label }}
        </SortableHeaderCell>
        <div v-if="col.draggable"
          class="absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-primary/50 transition-colors"
          :class="{ 'bg-primary/50': resizing === col.key }"
          @mousedown="handleMouseDown($event, col)" />
      </div>

      <div v-else-if="col.label" class="flex items-center px-2 truncate">{{ col.label }}</div>
    </template>
  </div>
</template>
