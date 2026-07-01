<script setup lang="ts">
import { usePlayerStore } from '../stores/player'
import { useAppStore } from '../stores/app'
import { formatTime } from '../lib/utils'
import PlayerControlButton from './player/PlayerControlButton.vue'
import MarqueeText from './MarqueeText.vue'
import TrackContextMenu from './TrackContextMenu.vue'
import Slider from './Slider.vue'
import { enterMiniMode, exitMiniMode } from '../composables/useInAppMiniMode'

const store = usePlayerStore()
const appStore = useAppStore()

const trackContextMenu = ref<InstanceType<typeof TrackContextMenu> | null>(null)

function openArtworkContextMenu(e: MouseEvent) {
  if (!store.currentTrack) return
  trackContextMenu.value?.open(e, store.currentTrack, { excludePlayNext: true })
}

const isSeeking = ref(false)
const seekValue = ref(0)

const displayPosition = computed(() =>
  isSeeking.value ? (seekValue.value / 100) * store.duration : store.position,
)

const repeatIcon = computed(() =>
  store.repeatMode === 'one' ? Repeat1 : Repeat,
)
const repeatActive = computed(
  () => store.repeatMode === 'one' || store.repeatMode === 'all',
)

function onSeekStart() {
  isSeeking.value = true
  seekValue.value = store.progressPercent
}

async function onSeekEnd() {
  await store.seek((seekValue.value / 100) * store.duration)
  isSeeking.value = false
}

async function onMiniToggle() {
  if (store.playerMode === 'mini') {
    store.playerMode = 'sticky'
    try { await exitMiniMode() } catch (e) { console.warn('exitMiniMode failed', e) }
  } else {
    store.playerMode = 'mini'
    try { await enterMiniMode(360, 88, false) } catch (e) { console.warn('enterMiniMode failed', e) }
  }
}
</script>

<template>
  <div
    class="h-[72px] bg-background border-t border-foreground/[0.06] flex items-center justify-between px-6 gap-6 select-none">
    <!-- Track Info -->
    <div class="flex items-center justify-start gap-3 w-1/4 min-w-[200px]">
      <div
        class="w-12 h-12 rounded-lg overflow-hidden flex-shrink-0 shadow-lg ring-1 ring-foreground/10 cursor-pointer transition-transform hover:scale-105 active:scale-95"
        @click="store.openTrackInfo(store.currentTrack)" @contextmenu.prevent="openArtworkContextMenu">
        <img v-if="store.artworkUrlSm" :src="store.artworkUrlSm" :alt="store.currentTrack?.title || 'Not Playing'"
          class="w-full h-full object-cover" />
        <div v-else class="w-full h-full bg-foreground/5 flex items-center justify-center">
          <Music class="w-5 h-5 text-foreground opacity-40" />
        </div>
      </div>
      <div class="flex flex-col min-w-0 flex-1">
        <MarqueeText :text="store.currentTrack?.title || 'Not Playing'" content-class="font-medium text-sm leading-tight" />
        <MarqueeText :text="store.currentTrack?.artist || ''" content-class="text-xs text-foreground opacity-60 leading-tight mt-0.5" />
      </div>
    </div>

    <!-- Playback Controls -->
    <div class="flex-1 flex flex-col items-center gap-2 max-w-[600px]">
      <div class="flex items-center gap-5">
        <PlayerControlButton
          class="transition-opacity"
          :class="store.shuffle ? 'text-primary opacity-100' : 'text-foreground opacity-60 hover:text-foreground opacity-50'"
          :active="store.shuffle"
          :show-indicator="appStore.showPlayerIndicator"
          @click="store.setShuffle(!store.shuffle)"
        >
          <Shuffle class="w-4 h-4" />
        </PlayerControlButton>
        <PlayerControlButton
          class="transition-colors"
          :class="repeatActive ? 'text-primary' : 'text-foreground opacity-60 hover:text-foreground opacity-50'"
          :active="repeatActive"
          :show-indicator="appStore.showPlayerIndicator"
          @click="store.cycleRepeat()"
        >
          <component :is="repeatIcon" class="w-4 h-4" />
        </PlayerControlButton>
      </div>

      <!-- Seek bar -->
      <div class="w-full flex items-center gap-2">
        <span class="text-[10px] text-foreground opacity-50 tabular-nums w-8 text-right">
          {{ formatTime(displayPosition) }}
        </span>
        <Slider :model-value="isSeeking ? seekValue : store.progressPercent" :min="0" :max="100" :step="0.1"
          class="flex-1" @update:model-value="(v: number) => (seekValue = v)" @mousedown="onSeekStart" @mouseup="onSeekEnd"
          @touchstart="onSeekStart" @touchend="onSeekEnd" />
        <span class="text-[10px] text-foreground opacity-50 tabular-nums w-8">
          {{ formatTime(store.duration) }}
        </span>
      </div>
    </div>

    <!-- Volume & Options -->
    <div class="flex items-center justify-end gap-4 w-1/4 min-w-[200px]">
      <div class="flex items-center gap-3 w-full max-w-[220px]">
        <button class="text-foreground/80 hover:text-foreground transition-colors flex-shrink-0" @click="store.toggleMute()">
          <VolumeX v-if="store.isMuted" class="w-4 h-4" />
          <Volume2 v-else class="w-4 h-4" />
        </button>
        <Slider :model-value="store.isMuted ? 0 : store.volume" :min="0" :max="1" :step="0.01" class="flex-1"
          @update:model-value="(v: number) => store.setVolume(v)" />
      </div>
      <button class="transition-colors"
        :class="store.isLyricsOpen ? 'text-primary' : 'text-foreground opacity-60 hover:text-foreground opacity-50'"
        @click="store.toggleLyrics()">
        <Mic2 class="w-4 h-4" />
      </button>
      <button class="transition-colors"
        :class="store.isQueueOpen ? 'text-primary' : 'text-foreground opacity-60 hover:text-foreground opacity-50'"
        @click="store.toggleQueue()">
        <ListMusic class="w-4 h-4" />
      </button>
      <button class="text-foreground opacity-60 hover:text-foreground opacity-50 transition-colors"
        @click="onMiniToggle()">
        <PictureInPicture2 class="w-4 h-4" />
      </button>
      <button class="text-foreground opacity-60 hover:text-foreground opacity-50 transition-colors"
        @click="store.playerMode = 'fullscreen'">
        <Maximize class="w-4 h-4" />
      </button>
    </div>
  </div>

  <TrackContextMenu ref="trackContextMenu" />
</template>

<style scoped>
</style>
