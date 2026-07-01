<script setup lang="ts">
import { ref, computed } from 'vue'
import { Music } from 'lucide-vue-next'
import { usePlayerStore } from '../stores/player'
import { useTrackTableSettings, COLUMN_MAP } from '../composables/useTrackTableSettings'
import type { ColumnKey } from '../composables/useTrackTableSettings'
import type { Track } from '../lib/invoke'
import TrackTableHeader from './TrackTableHeader.vue'
import TrackTableRow from './TrackTableRow.vue'
import TrackContextMenu from './TrackContextMenu.vue'

const SIMPLE_COLUMNS: ColumnKey[] = ['index', 'title', 'artist', 'duration', 'context_menu']

const props = withDefaults(defineProps<{
  tracks: Track[]
  isLoading?: boolean
  playlistId?: string
  variant?: 'default' | 'glass'
  scrollToCurrent?: boolean
  hideHeader?: boolean
  allowDnd?: boolean
  simpleMode?: boolean
  contextMenuOptions?: { showRemoveFromQueue?: boolean; playlistId?: string }
}>(), {
  variant: 'default',
  scrollToCurrent: false,
  hideHeader: false,
  allowDnd: false,
  simpleMode: false,
})

const emit = defineEmits<{
  playTrack: [track: Track, index: number]
  reorder: [tracks: Track[]]
  navigateAlbum: [name: string]
  navigateArtist: [name: string]
}>()

const playerStore = usePlayerStore()
const settings = useTrackTableSettings()
const trackContextMenu = ref<InstanceType<typeof TrackContextMenu> | null>(null)
const trackTableEl = ref<HTMLElement | null>(null)
const selectedIds = ref<Set<string>>(new Set())
const lastSelectedIndex = ref<number | null>(null)
const selectionAnchorIndex = ref<number | null>(null)
const sortColumn = ref<string | null>(null)
const sortDir = ref<'asc' | 'desc'>('asc')

const orderedVisibleColumns = computed(() => {
  if (props.simpleMode) {
    return SIMPLE_COLUMNS.map((k) => COLUMN_MAP[k])
  }
  const order = settings.columnOrder.value
  const visible = settings.visibleColumns.value
  return order.filter((k) => visible.includes(k)).map((k) => COLUMN_MAP[k])
})

const gridTemplateColumns = computed(() =>
  orderedVisibleColumns.value.map((c) =>
    props.simpleMode ? c.gridWidth : settings.effectiveGridWidth(c)
  ).join(' '),
)

const totalMinWidth = computed(() => {
  const sum = orderedVisibleColumns.value.reduce((acc, c) => acc + c.minWidthPx, 0)
  return sum + 'px'
})

const headerContainerRef = ref<HTMLElement | null>(null)

function onTableScroll(e: Event) {
  const target = e.target as HTMLElement
  if (headerContainerRef.value) {
    headerContainerRef.value.scrollLeft = target.scrollLeft
  }
}

const sortedTracks = computed(() => {
  if (!sortColumn.value) return props.tracks
  const col = COLUMN_MAP[sortColumn.value as ColumnKey]
  if (!col?.sortFn) return props.tracks
  return [...props.tracks].sort((a, b) => {
    return sortDir.value === 'asc' ? col.sortFn!(a, b) : -col.sortFn!(a, b)
  })
})

function handleSort(col: string) {
  if (sortColumn.value === col) {
    sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortColumn.value = col
    sortDir.value = 'asc'
  }
}

function handleReorderColumn(from: ColumnKey, to: ColumnKey) {
  settings.reorderColumns(from, to)
}

function handleResizeColumn(key: ColumnKey, width: number) {
  settings.setColumnWidth(key, width)
}

function handleTrackClick(e: MouseEvent, track: Track, index: number) {
  const cmdOrCtrl = e.metaKey || e.ctrlKey
  const shift = e.shiftKey
  if (shift && selectionAnchorIndex.value !== null) {
    const start = Math.min(selectionAnchorIndex.value, index)
    const end = Math.max(selectionAnchorIndex.value, index)
    const newSelected = new Set(cmdOrCtrl ? selectedIds.value : [])
    for (let i = start; i <= end; i++) newSelected.add(sortedTracks.value[i].id)
    selectedIds.value = newSelected
    lastSelectedIndex.value = index
  } else if (cmdOrCtrl) {
    const s = new Set(selectedIds.value)
    if (s.has(track.id)) s.delete(track.id); else s.add(track.id)
    selectedIds.value = s
    lastSelectedIndex.value = index
    selectionAnchorIndex.value = index
  } else {
    selectedIds.value = new Set([track.id])
    lastSelectedIndex.value = index
    selectionAnchorIndex.value = index
  }
}

function playTrack(track: Track, index: number) {
  playerStore.playTracks(sortedTracks.value, index)
  emit('playTrack', track, index)
}

function onTrackContextMenu(e: MouseEvent, track: Track, index: number) {
  if (selectedIds.value.size > 1 && selectedIds.value.has(track.id)) {
    const selectedTracks = sortedTracks.value.filter(t => selectedIds.value.has(t.id))
    trackContextMenu.value?.openMulti(e, selectedTracks, { playlistId: props.playlistId })
  } else {
    selectedIds.value = new Set([track.id])
    lastSelectedIndex.value = index
    selectionAnchorIndex.value = index
    trackContextMenu.value?.open(e, track, {
      excludePlayNext: false,
      showRemoveFromQueue: props.contextMenuOptions?.showRemoveFromQueue,
      playlistId: props.playlistId,
      onNavigateAlbum: (name: string) => emit('navigateAlbum', name),
      onNavigateArtist: (name: string) => emit('navigateArtist', name),
    })
  }
}

function scrollToCurrentTrack() {
  if (!trackTableEl.value) return
  const currentIdx = playerStore.queueIndex
  const rows = trackTableEl.value.querySelectorAll('[data-track-index]')
  const target = rows[currentIdx] as HTMLElement | undefined
  if (target) target.scrollIntoView({ behavior: 'smooth', block: 'center' })
}

const dragOverIndex = ref<number | null>(null)

function onDragOver(e: DragEvent) {
  if (!props.allowDnd) return
  e.preventDefault()
  const rows = trackTableEl.value?.querySelectorAll('[draggable]')
  if (!rows) return
  let closest = -1
  let closestOffset = Number.NEGATIVE_INFINITY
  rows.forEach((row, i) => {
    const rect = row.getBoundingClientRect()
    const offset = e.clientY - rect.top - rect.height / 2
    if (offset < 0 && offset > closestOffset) { closestOffset = offset; closest = i }
  })
  dragOverIndex.value = closest
}

function onDrop(e: DragEvent) {
  if (!props.allowDnd) return
  e.preventDefault()
  dragOverIndex.value = null
  try {
    const data = JSON.parse(e.dataTransfer?.getData('text/plain') || '{}')
    const fromIndex = data.index as number
    if (fromIndex === undefined) return
    const newTracks = [...sortedTracks.value]
    const [moved] = newTracks.splice(fromIndex, 1)
    newTracks.splice(fromIndex >= newTracks.length ? newTracks.length : fromIndex, 0, moved)
    emit('reorder', newTracks)
  } catch {}
}

function onDragLeave() { dragOverIndex.value = null }

defineExpose({ scrollToCurrentTrack })
</script>

<template>
  <div ref="trackTableEl"
    class="flex flex-col h-full overflow-hidden"
    :class="variant === 'glass' ? 'dark' : ''"
    @dragover="onDragOver" @drop="onDrop" @dragleave="onDragLeave">

    <div ref="headerContainerRef" v-if="!hideHeader" class="overflow-hidden flex-shrink-0">
      <div :style="{ minWidth: totalMinWidth }">
        <TrackTableHeader
          :grid-template-columns="gridTemplateColumns"
          :ordered-visible-columns="orderedVisibleColumns"
          :sort-column="sortColumn"
          :sort-dir="sortDir"
          :variant="variant"
          @sort="handleSort"
          @reorder-column="handleReorderColumn"
          @resize-column="handleResizeColumn" />
      </div>
    </div>

    <div class="flex-1 overflow-auto custom-scrollbar relative" @scroll="onTableScroll">
      <div v-if="isLoading" class="flex items-center justify-center py-12 text-foreground opacity-50 text-sm">Loading...</div>
      <div v-else-if="sortedTracks.length === 0"
        class="flex flex-col items-center justify-center py-16 gap-2 text-foreground/40">
        <Music class="w-8 h-8" />
        <span class="text-sm">No tracks found</span>
      </div>

      <div v-if="scrollToCurrent && sortedTracks.length > 0" class="flex items-center justify-center py-2">
        <button @click="scrollToCurrentTrack" class="text-xs text-foreground/40 hover:text-foreground transition-colors flex items-center gap-1">
          Scroll to current track
        </button>
      </div>

      <div :style="{ minWidth: totalMinWidth }">
        <TrackTableRow v-for="(track, i) in sortedTracks" :key="track.id" :track="track" :index="i"
          :is-selected="selectedIds.has(track.id)"
          :ordered-visible-columns="orderedVisibleColumns"
          :grid-template-columns="gridTemplateColumns"
          :draggable="allowDnd"
          :variant="variant"
          :collapsed="settings.collapsedMode.value"
          @play-track="playTrack"
          @click="(e) => handleTrackClick(e, track, i)"
          @contextmenu="(e) => onTrackContextMenu(e, track, i)"
          @navigate-album="(name) => emit('navigateAlbum', name)"
          @navigate-artist="(name) => emit('navigateArtist', name)" />
      </div>
    </div>
  </div>
  <TrackContextMenu ref="trackContextMenu" />
</template>
