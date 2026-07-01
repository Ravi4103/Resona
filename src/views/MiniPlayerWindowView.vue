<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  SkipBack, SkipForward, Play, Pause,
  Pin, PinOff, X, Music,
  Shuffle, Repeat, Repeat1,
  Volume2, VolumeX,
} from 'lucide-vue-next'
import LazyImg from '../components/LazyImg.vue'
import { usePlayerStore } from '../stores/player'
import { useAppStore } from '../stores/app'
import { formatTime, hexToRgba } from '../lib/utils'
import MarqueeText from '../components/MarqueeText.vue'
import Slider from '../components/Slider.vue'
import PlayerControlButton from '../components/player/PlayerControlButton.vue'
import { useGlassBlur } from '../composables/useGlassBlur'
import { closeMiniWindow } from '../composables/useMiniPlayerWindow'

const store = usePlayerStore()
const appStore = useAppStore()

const canvasRef = ref<HTMLCanvasElement | null>(null)
useGlassBlur(canvasRef, computed(() => store.artworkUrlMd ?? null))

const alwaysOnTop = ref(localStorage.getItem('resona-mini-always-on-top') === 'true')
const isSeeking = ref(false)
const seekValue = ref(0)
const isHovered = ref(false)
const showVolume = ref(false)
let volumeHideTimer: ReturnType<typeof setTimeout> | null = null

const displayPosition = computed(() =>
  isSeeking.value ? (seekValue.value / 100) * store.duration : store.position,
)

const trackTitle = computed(() => {
  if (!store.currentTrack) return 'Not Playing'
  return store.currentTrack.title || 'Not Playing'
})

const trackArtist = computed(() => store.currentTrack?.artist || '')

const repeatActive = computed(() => store.repeatMode === 'all' || store.repeatMode === 'one')
const repeatIcon = computed(() => store.repeatMode === 'one' ? Repeat1 : Repeat)

async function toggleAlwaysOnTop() {
  alwaysOnTop.value = !alwaysOnTop.value
  localStorage.setItem('resona-mini-always-on-top', String(alwaysOnTop.value))
  try {
    await getCurrentWindow().setAlwaysOnTop(alwaysOnTop.value)
  } catch (e) {
    console.error('Failed to set always on top:', e)
  }
}

function onSeekStart() { isSeeking.value = true }
async function onSeekEnd() {
  await store.seek((seekValue.value / 100) * store.duration)
  isSeeking.value = false
}

function onVolumeEnter() {
  if (volumeHideTimer) { clearTimeout(volumeHideTimer); volumeHideTimer = null }
  showVolume.value = true
}
function onVolumeLeave() {
  volumeHideTimer = setTimeout(() => { showVolume.value = false }, 300)
}

let offWindowFocus: (() => void) | null = null

onMounted(async () => {
  await store.init()
  await store.syncState()
  
  try {
    await getCurrentWindow().setAlwaysOnTop(alwaysOnTop.value)
  } catch (e) {
    console.warn('Could not set always on top on mount:', e)
  }
  
  offWindowFocus = await listen('tauri://focus', () => {
    store.syncState()
  })
})

onUnmounted(() => {
  if (volumeHideTimer) clearTimeout(volumeHideTimer)
  offWindowFocus?.()
})

watch(alwaysOnTop, async (val) => {
  try {
    await getCurrentWindow().setAlwaysOnTop(val)
  } catch (e) {
    console.error('Failed to update always on top:', e)
  }
}, { immediate: false })

watch(() => store.theme, (colors) => {
  if (!colors) return
  const root = document.documentElement
  root.style.setProperty('--dynamic-primary', colors.vibrant)
  root.style.setProperty('--dynamic-surface', hexToRgba(colors.dominant, 0.15))
  root.style.setProperty('--dynamic-glow', `0 0 40px ${hexToRgba(colors.vibrant, 0.3)}`)
})
</script>

<template>
  <div class="h-full w-full bg-[#0A0A0A] text-white overflow-hidden dark">
    <div class="relative w-full h-full overflow-hidden select-none" data-tauri-drag-region
      @mouseenter="isHovered = true" @mouseleave="isHovered = false">
      <!-- Artwork fills entire window -->
      <div class="absolute inset-0 bg-[#0A0A0A]" style="-webkit-app-region: no-drag">
        <LazyImg v-if="store.artworkUrl" :src="store.artworkUrl" :alt="trackTitle" class="w-full h-full object-cover" />
        <div v-else class="w-full h-full flex items-center justify-center bg-white/5">
          <Music class="w-16 h-16 text-white/20" />
        </div>
      </div>

      <!-- Options pill: top-right -->
      <div class="absolute top-2 right-2 z-30" style="-webkit-app-region: no-drag">
        <!-- Volume slider popup -->
        <Transition name="fade">
          <div v-if="showVolume && isHovered"
            class="absolute top-full right-0 mt-2 px-2.5 py-2 rounded-xl bg-black/20 backdrop-blur-md border border-white/5"
            @mouseenter="onVolumeEnter" @mouseleave="onVolumeLeave">
            <Slider 
              :model-value="store.isMuted ? 0 : store.volume * 100" 
              :min="0" 
              :max="100" 
              :step="1" 
              class="w-20"
              @update:model-value="(v: number) => store.setVolume(v / 100)" 
            />
          </div>
        </Transition>

        <div
          class="inline-flex items-center p-1 rounded-full bg-black/20 backdrop-blur-md border border-white/5 h-8 select-none"
          :class="isHovered ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'">
          <button
            class="w-6 h-6 flex items-center justify-center rounded-full text-white/50 hover:text-white/80 transition-colors"
            @mouseenter="onVolumeEnter" @mouseleave="onVolumeLeave" @click="showVolume = !showVolume">
            <VolumeX v-if="store.isMuted" class="w-3.5 h-3.5" />
            <Volume2 v-else class="w-3.5 h-3.5" />
          </button>
          <button class="w-6 h-6 flex items-center justify-center rounded-full transition-colors"
            :class="alwaysOnTop ? 'text-white/80' : 'text-white/50 hover:text-white/80'" @click="toggleAlwaysOnTop()">
            <Pin v-if="alwaysOnTop" class="w-3.5 h-3.5" />
            <PinOff v-else class="w-3.5 h-3.5" />
          </button>
          <button
            class="w-6 h-6 flex items-center justify-center rounded-full text-white/50 hover:text-white/80 transition-colors"
            @click="closeMiniWindow()">
            <X class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      <!-- WebGL glass blur panel -->
      <canvas
        ref="canvasRef"
        class="absolute bottom-0 left-0 pointer-events-none transition-opacity duration-200 blur-xl"
        :class="isHovered ? 'opacity-100' : 'opacity-0'"
        style="height: 250px; width: 500px;"
      />

      <!-- Controls overlay (hover-triggered) -->
      <div class="absolute bottom-0 left-0 right-0 px-3 pb-2 transition-opacity duration-200"
        :class="isHovered ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'"
        style="-webkit-app-region: no-drag">
        <MarqueeText :text="trackTitle" content-class="text font-semibold leading-tight text-white" />
        <MarqueeText :text="trackArtist" content-class="text-xs text-white/50 leading-tight mt-0.5" />

        <!-- Seek bar -->
        <div class="flex items-center gap-1.5 mt-2">
          <span class="text-[10px] text-white/40 tabular-nums w-7 text-right shrink-0">
            {{ formatTime(displayPosition) }}
          </span>
          <Slider 
            :model-value="isSeeking ? seekValue : store.progressPercent" 
            :min="0" 
            :max="100" 
            :step="0.1"
            class="flex-1" 
            @update:model-value="(v: number) => (seekValue = v)" 
            @mousedown="onSeekStart" 
            @mouseup="onSeekEnd" 
          />
          <span class="text-[10px] text-white/40 tabular-nums w-7 shrink-0">
            {{ formatTime(store.duration) }}
          </span>
        </div>

        <!-- Controls -->
        <div class="flex items-center justify-center gap-4 mt-2 mb-1">
          <PlayerControlButton
            class="transition-colors"
            :class="store.shuffle ? 'text-white/80' : 'text-white/20 hover:text-white/70'"
            :active="store.shuffle"
            :show-indicator="appStore.showPlayerIndicator"
            dot-class="bg-white"
            @click="store.setShuffle(!store.shuffle)">
            <Shuffle class="w-3.5 h-3.5" />
          </PlayerControlButton>
          <button class="text-white/80 hover:text-white/90 transition-colors" @click="store.previous()">
            <SkipBack class="w-4 h-4 fill-current" />
          </button>
          <button
            class="w-9 h-9 bg-white rounded-full flex items-center justify-center hover:scale-105 transition-transform shrink-0"
            @click="store.togglePlayPause()">
            <Pause v-if="store.isPlaying" class="w-4.5 h-4.5 fill-current text-[#0A0A0A]" />
            <Play v-else class="w-4.5 h-4.5 fill-current text-[#0A0A0A] ml-0.5" />
          </button>
          <button class="text-white/80 hover:text-white/90 transition-colors" @click="store.next()">
            <SkipForward class="w-4 h-4 fill-current" />
          </button>
          <PlayerControlButton
            class="transition-colors"
            :class="repeatActive ? 'text-white/80' : 'text-white/20 hover:text-white/70'"
            :active="repeatActive"
            :show-indicator="appStore.showPlayerIndicator"
            dot-class="bg-white"
            @click="store.cycleRepeat()">
            <component :is="repeatIcon" class="w-3.5 h-3.5" />
          </PlayerControlButton>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>