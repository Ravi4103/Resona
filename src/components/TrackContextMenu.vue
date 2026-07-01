<script setup lang="ts">
import type { Track } from '../lib/invoke'
import { useContextMenu } from '../composables/useContextMenu'
import { useTrackContextMenu, type TrackContextMenuOptions } from '../composables/useTrackContextMenu'
import ContextMenu from './ContextMenu.vue'
import FindLyricsDialog from './FindLyricsDialog.vue'
import MetadataEditDialog from './MetadataEditDialog.vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const contextMenu = useContextMenu()
const { buildMenuItems, buildMultiSelectMenuItems, editingTrack, metadataOpen, findLyricsOpen } = useTrackContextMenu()

function open(e: MouseEvent, track: Track, options?: TrackContextMenuOptions) {
  const mergedOptions: TrackContextMenuOptions = {
    ...options,
    onNavigateAlbum: (name: string) => router.push(`/albums/${encodeURIComponent(name)}`),
    onNavigateArtist: (name: string) => router.push(`/artists/${encodeURIComponent(name)}`),
  }
  contextMenu.open(e, buildMenuItems(track, mergedOptions))
}

function openMulti(e: MouseEvent, tracks: Track[], options?: TrackContextMenuOptions) {
  contextMenu.open(e, buildMultiSelectMenuItems(tracks, options))
}

function close() {
  contextMenu.close()
}

defineExpose({ open, openMulti, close })
</script>

<template>
  <ContextMenu
    :visible="contextMenu.visible.value"
    :x="contextMenu.x.value"
    :y="contextMenu.y.value"
    :items="contextMenu.items.value"
    @close="contextMenu.close()"
  />
  <FindLyricsDialog v-model:open="findLyricsOpen" :track="editingTrack" />
  <MetadataEditDialog v-model:open="metadataOpen" :track="editingTrack" />
</template>
