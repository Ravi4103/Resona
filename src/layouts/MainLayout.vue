<script setup lang="ts">
import { computed, ref } from 'vue'
import Sidebar from '../components/Sidebar.vue'
import PlayerFooter from '../components/PlayerFooter.vue'
import MiniPlayer from '../components/player/MiniPlayer.vue'
import QueueDrawer from '../components/QueueDrawer.vue'
import LyricsDrawer from '../components/LyricsDrawer.vue'
import TrackInfoDrawer from '../components/TrackInfoDrawer.vue'
import { RouterView } from 'vue-router'
import { usePlayerStore } from '../stores/player'
import { useAppStore } from '../stores/app'

import FullScreenPlayer from '../components/player/FullScreenPlayer.vue'

const playerStore = usePlayerStore()
const appStore = useAppStore()

const rightPanelOpen = computed(() => playerStore.isQueueOpen || playerStore.isLyricsOpen || playerStore.isTrackInfoOpen)
const rightPanelWidth = computed(() => rightPanelOpen.value ? 320 : 0)
const showSidebar = computed(() => playerStore.playerMode !== 'fullscreen')

const sidebarWidth = computed(() => appStore.sidebarWidth)

const mainStyle = computed(() => ({
  width: rightPanelOpen.value
    ? `calc(100% - ${sidebarWidth.value}px - ${rightPanelWidth.value}px)`
    : `calc(100% - ${sidebarWidth.value}px)`,
}))

const isResizing = ref(false)

function startResize(e: MouseEvent) {
  isResizing.value = true
  const startX = e.clientX
  const startWidth = sidebarWidth.value

  const onMove = (ev: MouseEvent) => {
    appStore.updateSidebarWidth(startWidth + (ev.clientX - startX))
  }
  const onUp = () => {
    isResizing.value = false
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
  }
  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}
</script>

<template>
  <div class="h-full w-full flex flex-col overflow-hidden bg-background text-foreground">
    <template v-if="playerStore.playerMode !== 'fullscreen'">
      <div class="flex-1 min-h-0 flex overflow-hidden">
        <aside v-if="showSidebar" :style="{ width: sidebarWidth + 'px' }"
          class="h-full overflow-hidden flex-shrink-0 select-none">
          <Sidebar class="pt-4" />
        </aside>
        <div
          v-if="showSidebar"
          class="w-1 flex-shrink-0 cursor-col-resize hover:bg-primary/30 transition-colors relative group"
          :class="{ 'bg-primary/40': isResizing }"
          @mousedown.prevent="startResize">
          <div class="absolute inset-y-0 -left-1 -right-1" />
        </div>
        <main class="flex-1 min-w-0 flex flex-col overflow-hidden" :style="mainStyle">
          <RouterView v-slot="{ Component }">
            <KeepAlive :max="5">
              <component :is="Component" />
            </KeepAlive>
          </RouterView>
        </main>
        <Transition name="drawer-slide">
          <div v-if="playerStore.isQueueOpen" class="w-80 flex-shrink-0 border-l border-foreground/[0.06] overflow-hidden">
            <QueueDrawer />
          </div>
          <div v-else-if="playerStore.isLyricsOpen" class="w-80 flex-shrink-0 border-l border-foreground/[0.06] overflow-hidden">
            <LyricsDrawer />
          </div>
          <div v-else-if="playerStore.isTrackInfoOpen" class="w-80 flex-shrink-0 border-l border-foreground/[0.06] overflow-hidden">
            <TrackInfoDrawer />
          </div>
        </Transition>
      </div>
      <PlayerFooter v-if="playerStore.playerMode === 'sticky'" />
      <MiniPlayer v-else-if="playerStore.playerMode === 'mini'" />
    </template>

    <!-- Fullscreen Player -->
    <FullScreenPlayer v-else @close="playerStore.playerMode = 'sticky'" />
  </div>
</template>

<style scoped>
.drawer-slide-enter-active, .drawer-slide-leave-active {
  transition: width 0.25s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.25s ease, transform 0.25s ease;
}
.drawer-slide-enter-from, .drawer-slide-leave-to {
  width: 0 !important;
  opacity: 0;
  transform: translateX(16px);
}
.drawer-slide-enter-to, .drawer-slide-leave-from {
  width: 320px !important;
  opacity: 1;
  transform: translateX(0);
}
.animate-slide-up { animation: slideUp 0.3s ease-out; }
@keyframes slideUp { from { transform: translateY(100%); } to { transform: translateY(0); } }
</style>
