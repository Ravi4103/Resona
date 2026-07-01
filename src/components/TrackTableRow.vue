<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Play, GripVertical, Heart, MoreHorizontal, Music } from 'lucide-vue-next'
import PlayingBar from './PlayingBar.vue'
import type { ColumnDef } from '../composables/useTrackTableSettings'
import { formatTime } from '../lib/utils'
import { usePlayerStore } from '../stores/player'
import { useFavoritesStore } from '../stores/favorites'
import * as api from '../lib/invoke'
import type { Track } from '../lib/invoke'

const props = withDefaults(defineProps<{
  track: Track
  index: number
  isSelected: boolean
  orderedVisibleColumns: ColumnDef[]
  gridTemplateColumns: string
  variant?: 'default' | 'glass'
  collapsed?: boolean
  draggable?: boolean
}>(), {
  variant: 'default',
  collapsed: false,
  draggable: false,
})

const emit = defineEmits<{
  playTrack: [track: Track, index: number]
  contextmenu: [e: MouseEvent, track: Track]
  click: [e: MouseEvent]
  navigateAlbum: [id: string]
  navigateArtist: [id: string]
}>()

const playerStore = usePlayerStore()
const favoritesStore = useFavoritesStore()
const rowRef = ref<HTMLElement | null>(null)
const artworkSrc = ref<string | null>(null)
let observer: IntersectionObserver | null = null

onMounted(() => {
  if (props.track.has_artwork) {
    observer = new IntersectionObserver((entries) => {
      entries.forEach(async (entry) => {
        if (entry.isIntersecting) {
          try { const b64 = await api.getArtwork(props.track.id); if (b64) artworkSrc.value = `data:image/jpeg;base64,${b64}` } catch {}
          if (observer && rowRef.value) { observer.unobserve(rowRef.value); observer.disconnect(); observer = null }
        }
      })
    }, { rootMargin: '100px' })
    if (rowRef.value) observer.observe(rowRef.value)
  }
})

onUnmounted(() => { if (observer && rowRef.value) { observer.unobserve(rowRef.value); observer.disconnect() } })

function isCurrent() { return playerStore.currentTrack?.id === props.track.id }
</script>

<template>
  <div ref="rowRef"
    class="group transition-colors select-none grid items-center text-sm"
    :class="[
      collapsed ? 'h-9 text-xs' : 'min-h-[56px]',
      variant === 'glass' ? 'hover:bg-white/[0.04]' : 'hover:bg-foreground/[0.04]',
      isCurrent() ? 'bg-primary/10 hover:bg-primary/[0.15]' : isSelected ? 'bg-foreground/[0.06]' : '',
    ]"
    :style="{ gridTemplateColumns: props.gridTemplateColumns }"
    :draggable="draggable"
    @click="emit('click', $event)"
    @dblclick="emit('playTrack', track, index)"
    @contextmenu.prevent="emit('contextmenu', $event, track)">

    <template v-for="col in orderedVisibleColumns" :key="col.key">
      <div v-if="col.key === 'dnd'" class="flex items-center justify-center cursor-grab text-foreground/30 hover:text-foreground/60">
        <GripVertical class="w-4 h-4" />
      </div>

      <div v-else-if="col.key === 'index'" class="flex items-center justify-center text-foreground/40 text-xs tabular-nums">
        <span v-if="!isCurrent()" class="hidden group-hover:hidden">{{ index + 1 }}</span>
        <button v-if="!isCurrent()" class="hidden group-hover:flex text-primary hover:scale-110 transition-transform"
          @click.stop="emit('playTrack', track, index)">
          <Play class="w-4 h-4 fill-current" />
        </button>
        <PlayingBar v-if="isCurrent()" :is-playing="playerStore.isPlaying" />
      </div>

      <div v-else-if="col.key === 'title'" class="font-medium truncate flex items-center gap-3 min-w-0 px-2">
        <div class="w-8 h-8 rounded flex-shrink-0 overflow-hidden bg-foreground/5 flex items-center justify-center">
          <img v-if="artworkSrc" :src="artworkSrc" :alt="track.title" class="w-full h-full object-cover" />
          <Music v-else class="w-4 h-4 text-foreground opacity-40" />
        </div>
        <span class="truncate" :class="{ 'text-primary': isCurrent() }">{{ track.title || 'Unknown' }}</span>
      </div>

      <div v-else-if="col.key === 'duration'" class="text-center text-foreground opacity-80 text-xs px-2 tabular-nums">{{ formatTime(track.duration) }}</div>

      <div v-else-if="col.key === 'artist'" class="text-foreground opacity-80 truncate flex items-center min-w-0 px-2">
        <div class="truncate">
          <span class="cursor-pointer hover:text-primary transition-colors" @click.stop="emit('navigateArtist', track.artist)">{{ track.artist || '-' }}</span>
        </div>
      </div>

      <div v-else-if="col.key === 'album'" class="text-foreground opacity-80 truncate flex items-center min-w-0 px-2">
        <span class="truncate cursor-pointer hover:text-primary transition-colors" @click.stop="emit('navigateAlbum', track.album)">{{ track.album || '-' }}</span>
      </div>

      <div v-else-if="col.key === 'year'" class="text-foreground/40 text-xs flex items-center">{{ track.year || '-' }}</div>
      <div v-else-if="col.key === 'genre'" class="text-foreground/40 truncate flex items-center text-xs">{{ track.raw_genre_names || track.genre || '-' }}</div>

      <div v-else-if="col.key === 'favorite'" class="flex items-center justify-center">
        <button @click.stop="favoritesStore.toggle(track.id)" class="transition-colors"
          :class="favoritesStore.isFavorite(track.id) ? 'text-favorite' : 'text-foreground/30 hover:text-foreground/60'">
          <Heart class="w-4 h-4" :fill="favoritesStore.isFavorite(track.id) ? 'currentColor' : 'none'" />
        </button>
      </div>

      <div v-else-if="col.key === 'play_count'" class="text-foreground/40 text-xs flex items-center">-</div>
      <div v-else-if="col.key === 'disc_number'" class="text-foreground/40 text-xs flex items-center">{{ track.disc_number || '-' }}</div>
      <div v-else-if="col.key === 'track_number'" class="text-foreground/40 text-xs flex items-center">{{ track.track_number || '-' }}</div>
      <div v-else-if="col.key === 'album_artist'" class="text-foreground/50 truncate flex items-center text-xs">{{ track.album_artist || '-' }}</div>

      <div v-else-if="col.key === 'context_menu'" class="flex items-center justify-center">
        <button @click.stop="emit('contextmenu', $event, track)"
          class="p-1 rounded hover:bg-foreground/10 transition-colors text-foreground/40 hover:text-foreground/70 opacity-0 group-hover:opacity-100">
          <MoreHorizontal class="w-4 h-4" />
        </button>
      </div>
    </template>
  </div>
</template>
