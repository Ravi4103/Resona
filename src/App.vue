<script setup lang="ts">
import { onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import MainLayout from './layouts/MainLayout.vue'
import { usePlayerStore } from './stores/player'
import { useAppStore } from './stores/app'
import { useDeviceStore } from './stores/device'
import { useLibraryStore } from './stores/library'
import { useFavoritesStore } from './stores/favorites'
import { useLibrarySync } from './composables/useLibrarySync'
import { applyThemeColors } from './lib/artwork'

const router = useRouter()
const playerStore = usePlayerStore()
const appStore = useAppStore()
const deviceStore = useDeviceStore()
const libraryStore = useLibraryStore()
const favoritesStore = useFavoritesStore()

useLibrarySync()

const handleKeyDown = (e: KeyboardEvent) => {
  const target = e.target as HTMLElement
  if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) return

  const isMac = deviceStore.isMac
  const ctrlKey = isMac ? e.metaKey : e.ctrlKey
  const altKey = e.altKey

  if (e.code === 'Space') {
    e.preventDefault()
    playerStore.togglePlayPause()
  } else if (ctrlKey && e.key === 'ArrowRight') {
    e.preventDefault()
    if (altKey) playerStore.fastForward()
    else playerStore.next()
  } else if (ctrlKey && e.key === 'ArrowLeft') {
    e.preventDefault()
    if (altKey) playerStore.rewind()
    else playerStore.previous()
  } else if (ctrlKey && e.key === 'ArrowUp') {
    e.preventDefault()
    playerStore.increaseVolume()
  } else if (ctrlKey && e.key === 'ArrowDown') {
    e.preventDefault()
    if (altKey) playerStore.toggleMute()
    else playerStore.decreaseVolume()
  } else if (ctrlKey && e.key.toLowerCase() === 's') {
    e.preventDefault()
    playerStore.setShuffle(!playerStore.shuffle)
  } else if (ctrlKey && e.key.toLowerCase() === 'r') {
    e.preventDefault()
    playerStore.cycleRepeat()
  } else if (ctrlKey && e.key.toLowerCase() === 'f') {
    e.preventDefault()
    router.push('/search')
  } else if (ctrlKey && e.key === ',') {
    e.preventDefault()
    router.push('/settings')
  }
}

onMounted(async () => {
  appStore.loadSavedTheme()
  appStore.loadSavedColors()
  appStore.loadSavedSettings()
  deviceStore.init()
  playerStore.init()
  favoritesStore.load()
  await libraryStore.loadAll()
  window.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown)
  playerStore.dispose()
})

watch(() => playerStore.theme, (colors) => applyThemeColors(colors))
watch(() => appStore.theme, () => applyThemeColors(playerStore.theme))
</script>

<template>
  <MainLayout />
</template>
