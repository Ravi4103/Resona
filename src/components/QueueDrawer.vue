<script setup lang="ts">
import { ref } from 'vue'
import { Music, X, ListMusic, GripVertical, MoreVertical, ArrowDownToLine } from 'lucide-vue-next'
import { formatTime } from '../lib/utils'
import { usePlayerStore } from '../stores/player'
import type { Track } from '../lib/invoke'
import TrackContextMenu from './TrackContextMenu.vue'

const store = usePlayerStore()
const trackContextMenu = ref<InstanceType<typeof TrackContextMenu> | null>(null)
const virtualRef = ref<any>(null)
const scrollerRef = ref<HTMLElement | null>(null)

function onContextMenu(e: MouseEvent, track: Track) {
  trackContextMenu.value?.open(e, track, { showRemoveFromQueue: true })
}

function scrollToCurrent() {
  if (store.currentTrack) virtualRef.value?.scrollToKey(store.currentTrack.id)
}

function onDrop(e: { newIndex: number; oldIndex: number }) {
  if (e.oldIndex !== e.newIndex) {
    const q = [...store.queue]
    const [moved] = q.splice(e.oldIndex, 1)
    q.splice(e.newIndex, 0, moved)
    store.queue = q
    if (store.queueIndex === e.oldIndex) store.queueIndex = e.newIndex
  }
}
</script>

<template>
  <div class="h-full w-full bg-background/80 backdrop-blur-2xl flex flex-col">
    <div class="flex items-center justify-between px-4 py-3 border-b border-foreground/[0.06]">
      <div class="flex items-center gap-2 font-semibold">
        <ListMusic class="w-4 h-4 text-primary" />
        <span>Queue</span>
        <span class="text-xs text-foreground opacity-50 font-normal ml-1">({{ store.queue.length }})</span>
      </div>
      <div class="flex items-center gap-1">
        <button @click="scrollToCurrent"
          class="p-1.5 rounded-full hover:bg-foreground/10 transition-colors text-foreground opacity-60 hover:text-foreground"
          title="Scroll to current">
          <ArrowDownToLine class="w-4 h-4" />
        </button>
        <button @click="store.toggleQueue()"
          class="p-1.5 rounded-full hover:bg-foreground/10 transition-colors text-foreground opacity-60 hover:text-foreground"
          title="Close">
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>

    <div ref="scrollerRef" class="flex-1 overflow-hidden">
      <div v-if="store.queue.length === 0" class="h-full flex flex-col items-center justify-center text-foreground opacity-50 gap-3">
        <Music class="w-10 h-10 opacity-20" />
        <p class="text-sm">Queue is empty</p>
      </div>
      <VirtualList v-else ref="virtualRef" :model-value="store.queue" :data-key="'id'" :size="56" :keeps="30"
        :sortable="true" :scroller="scrollerRef" root-tag="div" wrap-tag="div"
        @update:model-value="(val: any[]) => store.queue = val" @drop="onDrop">
        <template #item="{ record: track, index: i }">
          <div
            :data-current="store.currentTrack?.id === (track as Track).id ? true : null"
            class="flex items-center gap-2 px-4 py-2 hover:bg-foreground/[0.04] transition-colors cursor-pointer group select-none h-14"
            :class="{ 'bg-primary/10 border-l-2 border-l-primary': store.currentTrack?.id === (track as Track).id }"
            draggable="true"
            @dblclick="store.playTracks(store.queue, i)"
            @contextmenu.prevent="onContextMenu($event, track as Track)">
            <div class="opacity-20 group-hover:opacity-60 transition-opacity cursor-grab text-foreground flex-shrink-0">
              <GripVertical class="w-4 h-4" />
            </div>
            <div class="w-10 h-10 bg-foreground/5 rounded-md flex-shrink-0 overflow-hidden flex items-center justify-center">
              <Music class="w-4 h-4 text-foreground opacity-40" />
            </div>
            <div class="flex-1 min-w-0">
              <div class="text-sm truncate font-medium" :class="{ 'text-primary': store.currentTrack?.id === (track as Track).id }">
                {{ (track as Track).title || 'Unknown' }}
              </div>
              <div class="text-xs text-foreground opacity-50 truncate">{{ (track as Track).artist || 'Unknown Artist' }}</div>
            </div>
            <span class="text-xs text-foreground opacity-40 tabular-nums flex-shrink-0">{{ formatTime((track as Track).duration) }}</span>
            <button @click.stop="onContextMenu($event, track as Track)"
              class="p-1 opacity-0 group-hover:opacity-100 hover:bg-foreground/[0.08] rounded-full transition-all ml-1">
              <MoreVertical class="w-3.5 h-3.5 text-foreground opacity-50" />
            </button>
          </div>
        </template>
      </VirtualList>
    </div>
  </div>
  <TrackContextMenu ref="trackContextMenu" />
</template>
