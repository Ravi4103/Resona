<script setup lang="ts">
import { ref, watch } from 'vue'
import { ListMusic, X, Goal, Music, GripVertical, MoreVertical } from 'lucide-vue-next'
import { usePlayerStore } from '../../stores/player'
import { formatTime } from '../../lib/utils'
import TrackContextMenu from '../TrackContextMenu.vue'
import type { Track } from '../../lib/invoke'
import * as api from '../../lib/invoke'

defineProps<{
  queue: Track[]
}>()

const emit = defineEmits<{
  close: []
  playTrack: [index: number]
}>()

const playerStore = usePlayerStore()
const trackContextMenu = ref<InstanceType<typeof TrackContextMenu> | null>(null)
const listEl = ref<HTMLElement | null>(null)

const artworkCache = ref<Record<string, string>>({})

watch(() => playerStore.queue, async (tracks) => {
  for (const track of tracks) {
    if (track.has_artwork && !artworkCache.value[track.id]) {
      try {
        const b64 = await api.getArtwork(track.id)
        if (b64) artworkCache.value[track.id] = `data:image/jpeg;base64,${b64}`
      } catch {}
    }
  }
}, { immediate: true })

function scrollToCurrent() {
  if (!listEl.value) return
  const el = listEl.value.querySelector('[data-current]')
  if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' })
}

function onContextMenu(e: MouseEvent, track: Track) {
  trackContextMenu.value?.open(e, track, { showRemoveFromQueue: true })
}

const dragIndex = ref<number | null>(null)

function onDragStart(i: number) { dragIndex.value = i }
function onDragOver(e: DragEvent, i: number) { e.preventDefault(); dragIndex.value = i }
function onDrop(e: DragEvent, i: number) {
  e.preventDefault()
  if (dragIndex.value !== null && dragIndex.value !== i) {
    const q = [...playerStore.queue]
    const [moved] = q.splice(dragIndex.value, 1)
    q.splice(i, 0, moved)
    playerStore.reorderQueue(q)
    if (playerStore.queueIndex === dragIndex.value) playerStore.queueIndex = i
  }
  dragIndex.value = null
}
</script>

<template>
  <div class="absolute left-0 top-[2%] bottom-[8%] bg-black/30 backdrop-blur-3xl rounded-3xl border border-white/10 flex flex-col overflow-hidden shadow-2xl w-[50cqw] max-w-2xl">
    <div class="flex-1 flex flex-col h-full">
      <div class="flex items-center justify-between px-6 py-4 border-b border-white/5">
        <div class="flex items-center gap-2 text-white/80">
          <ListMusic class="w-4 h-4" />
          <span class="text-sm font-semibold uppercase tracking-wider">Up Next</span>
        </div>
        <div class="flex items-center gap-1">
          <button @click="scrollToCurrent"
            class="text-white/40 hover:text-white transition-colors p-1 hover:bg-white/5 rounded-full">
            <Goal class="w-4 h-4" />
          </button>
          <button @click="emit('close')"
            class="text-white/40 hover:text-white transition-colors p-1 hover:bg-white/5 rounded-full">
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <div ref="listEl" class="flex-1 overflow-y-auto custom-scrollbar px-2 py-2">
        <div v-if="queue.length === 0" class="flex flex-col items-center justify-center h-full text-white/40 gap-3">
          <ListMusic class="w-10 h-10 opacity-20" />
          <p class="text-sm">Queue is empty</p>
        </div>
        <div v-for="(track, i) in queue" :key="track.id"
          :data-current="playerStore.currentTrack?.id === track.id ? true : null"
          class="flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-white/5 transition-colors cursor-pointer group select-none"
          :class="{ 'bg-white/10 border-l-2 border-l-primary': playerStore.currentTrack?.id === track.id }"
          draggable="true"
          @dragstart="onDragStart(i)"
          @dragover.prevent="onDragOver($event, i)"
          @drop="onDrop($event, i)"
          @dblclick="emit('playTrack', i)"
          @contextmenu.prevent="onContextMenu($event, track)">
          <div class="opacity-20 group-hover:opacity-60 transition-opacity cursor-grab text-white flex-shrink-0">
            <GripVertical class="w-4 h-4" />
          </div>
          <div class="w-10 h-10 bg-white/5 rounded-md flex-shrink-0 overflow-hidden flex items-center justify-center">
            <img v-if="artworkCache[track.id]" :src="artworkCache[track.id]" :alt="track.title" class="w-full h-full object-cover" />
            <Music v-else class="w-4 h-4 text-white/40" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-sm truncate font-medium text-white"
              :class="{ 'text-primary': playerStore.currentTrack?.id === track.id }">
              {{ track.title || 'Unknown' }}
            </div>
            <div class="text-xs text-white/50 truncate">{{ track.artist || 'Unknown Artist' }}</div>
          </div>
          <span class="text-xs text-white/40 tabular-nums flex-shrink-0">{{ formatTime(track.duration) }}</span>
          <button @click.stop="onContextMenu($event, track)"
            class="p-1 opacity-0 group-hover:opacity-100 hover:bg-white/10 rounded-full transition-all ml-1">
            <MoreVertical class="w-3.5 h-3.5 text-white/50" />
          </button>
        </div>
      </div>
    </div>
  </div>
  <TrackContextMenu ref="trackContextMenu" />
</template>
