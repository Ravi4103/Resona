<script setup lang="ts">
import { Disc } from 'lucide-vue-next'
import { useLibraryStore } from '../stores/library'
import { useAlbumContextMenu } from '../composables/useAlbumContextMenu'
import { useContextMenu } from '../composables/useContextMenu'
import SortableHeaderCell from './SortableHeaderCell.vue'
import ContextMenu from './ContextMenu.vue'

type SortCol = 'title' | 'artist' | 'year'
type SortDir = 'asc' | 'desc'

const props = defineProps<{
  albums: { name: string; artist: string; year: number; trackCount: number; firstTrackId: string }[]
  sortColumn: SortCol
  sortDir: SortDir
}>()

const emit = defineEmits<{
  click: [name: string]
  play: [name: string]
  artistClick: [artist: string]
  'update:sortColumn': [col: SortCol]
  'update:sortDir': [dir: SortDir]
}>()

const library = useLibraryStore()
const contextMenu = useContextMenu()
const { buildMenuItems } = useAlbumContextMenu()

function cycleSort(col: SortCol) {
  if (props.sortColumn === col) { emit('update:sortDir', props.sortDir === 'asc' ? 'desc' : 'asc') }
  else { emit('update:sortColumn', col); emit('update:sortDir', 'asc') }
}

function onContextMenu(e: MouseEvent, album: { name: string; artist: string; year: number; trackCount: number; firstTrackId: string }) {
  const tracks = library.tracks.filter(t => t.album === album.name)
  contextMenu.open(e, buildMenuItems(tracks))
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden">
    <div class="grid grid-cols-[40px_1fr_1fr_80px_60px] gap-2 px-3 py-2 text-xs font-medium text-foreground opacity-50 border-b border-foreground/[0.06] select-none flex-shrink-0">
      <div class="flex items-center justify-center">#</div>
      <SortableHeaderCell :active="sortColumn === 'title'" :dir="sortColumn === 'title' ? sortDir : 'asc'" @click="cycleSort('title')">
        Title
      </SortableHeaderCell>
      <SortableHeaderCell :active="sortColumn === 'artist'" :dir="sortColumn === 'artist' ? sortDir : 'asc'" @click="cycleSort('artist')">
        Artist
      </SortableHeaderCell>
      <SortableHeaderCell :active="sortColumn === 'year'" :dir="sortColumn === 'year' ? sortDir : 'asc'" @click="cycleSort('year')">
        Year
      </SortableHeaderCell>
      <div class="flex items-center justify-end">Tracks</div>
    </div>

    <div class="flex-1 overflow-y-auto custom-scrollbar">
      <div v-for="album in albums" :key="album.name"
        class="grid grid-cols-[40px_1fr_1fr_80px_60px] gap-2 px-3 py-3 text-sm hover:bg-foreground/[0.04] group transition-colors select-none cursor-pointer"
        @click="emit('click', album.name)"
        @dblclick="emit('play', album.name)"
        @contextmenu.prevent="onContextMenu($event, album)">
        <div class="flex items-center justify-center">
          <div class="w-8 h-8 bg-foreground/5 rounded flex-shrink-0 overflow-hidden flex items-center justify-center">
            <Disc class="w-4 h-4 text-foreground opacity-40" />
          </div>
        </div>
        <div class="font-medium truncate flex items-center">{{ album.name || 'Unknown Album' }}</div>
        <div class="text-foreground opacity-70 truncate flex items-center">
          <span class="truncate hover:text-primary cursor-pointer transition-colors" @click.stop="emit('artistClick', album.artist || '')">{{ album.artist || 'Unknown Artist' }}</span>
        </div>
        <div class="text-foreground opacity-50 text-xs flex items-center justify-center tabular-nums">{{ album.year || '' }}</div>
        <div class="text-foreground opacity-50 text-xs flex items-center justify-end tabular-nums">{{ album.trackCount }}</div>
      </div>
    </div>
  </div>
  <ContextMenu :visible="contextMenu.visible.value" :x="contextMenu.x.value" :y="contextMenu.y.value"
    :items="contextMenu.items.value" @close="contextMenu.close()" />
</template>
