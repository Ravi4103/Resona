<script setup lang="ts">
import { computed, ref } from 'vue'
import { ListMusic, Mic2, Minimize2, Volume2, VolumeX } from 'lucide-vue-next'
import TabSwitcher from '../TabSwitcher.vue'
import type { TabOption } from '../TabSwitcher.vue'
import { usePlayerStore } from '../../stores/player'
import { useAppStore } from '../../stores/app'
import LivingArtworkBackground from './LivingArtworkBackground.vue'
import PlayerArtwork from './PlayerArtwork.vue'
import PlayerTrackInfo from './PlayerTrackInfo.vue'
import PlayerSeekBar from './PlayerSeekBar.vue'
import PlayerPlaybackControls from './PlayerPlaybackControls.vue'
import PlayerQueuePanel from './PlayerQueuePanel.vue'
import PlayerLyricsPanel from './PlayerLyricsPanel.vue'
import Slider from '../Slider.vue'
import TrackContextMenu from '../TrackContextMenu.vue'

const player = usePlayerStore()
const app = useAppStore()
const trackContextMenu = ref<InstanceType<typeof TrackContextMenu> | null>(null)

function openContextMenu(e: MouseEvent) {
  if (!player.currentTrack) return
  trackContextMenu.value?.open(e, player.currentTrack, { excludePlayNext: true })
}

function openTrackInfo() {
  if (!player.currentTrack) return
  if (player.playerMode === 'fullscreen') {
    player.playerMode = 'sticky'
  }
  player.openTrackInfo(player.currentTrack)
}

const activeTab = computed({
  get: () => {
    if (player.isLyricsOpen) return 'lyrics'
    if (player.isQueueOpen) return 'queue'
    return null
  },
  set: (val: string | null) => {
    if (val === 'lyrics') {
      player.isLyricsOpen = true
      player.isQueueOpen = false
    } else if (val === 'queue') {
      player.isQueueOpen = true
      player.isLyricsOpen = false
    } else {
      player.isQueueOpen = false
      player.isLyricsOpen = false
    }
  },
})

const showRightColumn = computed(() => player.isQueueOpen || player.isLyricsOpen)

const tabOptions: TabOption[] = [
  { value: 'lyrics', icon: Mic2, label: 'Lyrics' },
  { value: 'queue', icon: ListMusic, label: 'Queue' },
]
</script>

<template>
  <div class="fixed inset-0 z-100 flex flex-col overflow-hidden bg-[#0A0A0A] select-none dark">
    <LivingArtworkBackground />

    <div class="relative z-10 flex flex-col h-full text-white">
      <!-- Top bar -->
      <div class="flex items-center justify-between px-6 py-4">
        <div class="w-[120px]">
          <button class="p-2 rounded-full hover:bg-white/8 transition-all text-white/60 hover:text-white"
            @click="player.playerMode = 'sticky'">
            <Minimize2 class="w-5 h-5" />
          </button>
        </div>
        <span class="text-xs font-semibold text-white/40 uppercase tracking-[0.2em]">
          Now Playing
        </span>
        <div class="flex items-center gap-2 w-[120px] justify-end">
          <TabSwitcher v-model="activeTab" :options="tabOptions" />
        </div>
      </div>

      <!-- Main content -->
      <div class="flex-1 flex items-center justify-center px-8 w-full max-w-[1400px] mx-auto overflow-hidden">
        <div
          class="flex-1 flex flex-row items-center justify-center h-full transition-all duration-500 ease-[cubic-bezier(0.4,0,0.2,1)] relative"
          :class="showRightColumn ? 'gap-12' : 'gap-0'">
          <!-- Left Column: Cover and Controls -->
          <div
            class="flex flex-col items-center justify-center transition-all duration-500 ease-[cubic-bezier(0.4,0,0.2,1)]"
            :class="showRightColumn ? 'w-1/2 max-w-md' : 'w-full max-w-lg'">
            <div class="flex flex-col items-center justify-center gap-[clamp(0.75rem,2.5vh,1.5rem)] w-full min-h-0">
              <!-- Artwork -->
              <div @click="openTrackInfo" @contextmenu.prevent="openContextMenu">
                <PlayerArtwork :artwork-url="player.artworkUrl" :track-title="player.currentTrack?.title || 'Not Playing'"
                  :is-playing="player.isPlaying" size="lg" class="shadow-2xl ring-1 ring-foreground/10" />
              </div>

              <!-- Track info -->
              <div class="cursor-pointer" @click="openTrackInfo" @contextmenu.prevent="openContextMenu">
                <PlayerTrackInfo :title="player.currentTrack?.title || 'Not Playing'"
                  :artist="player.currentTrack?.artist || ''" :album="player.currentTrack?.album || ''" />
              </div>

              <!-- Seek bar -->
              <PlayerSeekBar :progress-percent="player.progressPercent" :position="player.position"
                :duration="player.duration" @seek="(v) => player.seek(v)" />

              <!-- Controls -->
              <PlayerPlaybackControls :is-playing="player.isPlaying" :shuffle="player.shuffle"
                :repeat-mode="player.repeatMode" :show-indicator="app.showPlayerIndicator"
                @toggle-play="player.togglePlayPause()" @next="player.next()"
                @previous="player.previous()" @toggle-shuffle="player.setShuffle(!player.shuffle)"
                @cycle-repeat="player.cycleRepeat()" />

              <!-- Volume -->
              <div class="flex items-center gap-3 w-full max-w-[220px]">
                <button class="text-white/80 hover:text-white transition-colors flex-shrink-0"
                  @click="player.toggleMute()">
                  <VolumeX v-if="player.isMuted" class="w-4 h-4" />
                  <Volume2 v-else class="w-4 h-4" />
                </button>
                <Slider :model-value="player.isMuted ? 0 : player.volume" :min="0" :max="1" :step="0.01" class="flex-1"
                  @update:model-value="(v: number) => player.setVolume(v)" />
              </div>
            </div>
          </div>

          <!-- Right Column -->
          <div
            class="h-full transition-all duration-500 ease-[cubic-bezier(0.4,0,0.2,1)] relative flex items-center justify-center"
            :class="showRightColumn ? 'w-1/2 max-w-2xl' : 'w-0'">
            <div v-if="player.isQueueOpen" class="w-full h-full relative">
              <PlayerQueuePanel :queue="player.queue"
                @close="player.isQueueOpen = false; player.isLyricsOpen = false"
                @play-track="(index) => player.playTracks(player.queue, index)" />
            </div>
            <div v-else-if="player.isLyricsOpen" class="w-full h-full relative">
              <PlayerLyricsPanel :lyrics="player.lyricsContent" :loading="false" :position="player.position"
                @close="player.isQueueOpen = false; player.isLyricsOpen = false"
                @seek="(time) => player.seek(time)" />
            </div>
          </div>
        </div>
      </div>
    </div>

    <TrackContextMenu ref="trackContextMenu" />
  </div>
</template>
