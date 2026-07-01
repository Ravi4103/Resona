import { ref } from 'vue'
import { CornerUpRight, Info, RefreshCw, Search, Heart, HeartOff, ListPlus, Disc, User, Pencil, FolderOpen, Check, ListX, Trash2 } from 'lucide-vue-next'
import type { ContextMenuItem } from './useContextMenu'
import type { Track } from '../lib/invoke'
import { usePlayerStore } from '../stores/player'
import { useFavoritesStore } from '../stores/favorites'
import { useLibraryStore } from '../stores/library'
import * as api from '../lib/invoke'

export interface TrackContextMenuOptions {
  excludePlayNext?: boolean
  showRemoveFromQueue?: boolean
  playlistId?: string
  onNavigateAlbum?: (name: string) => void
  onNavigateArtist?: (name: string) => void
}

export function useTrackContextMenu() {
  const playerStore = usePlayerStore()
  const favoritesStore = useFavoritesStore()
  const library = useLibraryStore()
  const editingTrack = ref<Track | null>(null)
  const metadataOpen = ref(false)
  const findLyricsOpen = ref(false)

  function buildMenuItems(track: Track, options: TrackContextMenuOptions = {}): ContextMenuItem[] {
    const items: ContextMenuItem[] = []
    const isCurrentTrack = playerStore.currentTrack?.id === track.id

    if (options.showRemoveFromQueue && !isCurrentTrack) {
      items.push({
        label: 'Remove from Queue',
        icon: ListX,
        action: () => {
          const idx = playerStore.queue.findIndex(t => t.id === track.id)
          if (idx >= 0) {
            const q = [...playerStore.queue]
            q.splice(idx, 1)
            playerStore.queue = q
            if (idx < playerStore.queueIndex) playerStore.queueIndex--
          }
        },
      })
      items.push({ separator: true })
    }

    if (!options.excludePlayNext && !isCurrentTrack) {
      items.push({
        label: 'Play Next',
        icon: CornerUpRight,
        action: () => {
          const q = [...playerStore.queue]
          q.splice(playerStore.queueIndex + 1, 0, track)
          playerStore.queue = q
        },
      })
    }

    items.push({
      label: 'Track Info',
      icon: Info,
      action: () => { playerStore.openTrackInfo(track) },
    })

    items.push({
      label: 'Refresh Lyrics',
      icon: RefreshCw,
      action: async () => {
        if (track.file_path) {
          try {
            const lrc = await api.getLyrics(track.file_path)
            if (playerStore.currentTrack?.id === track.id) {
              playerStore.lyricsContent = lrc ?? undefined
            }
          } catch {}
        }
      },
    })

    items.push({
      label: 'Find Lyrics',
      icon: Search,
      action: () => { editingTrack.value = track; findLyricsOpen.value = true },
    })

    items.push({ separator: true })

    const isFav = favoritesStore.isFavorite(track.id)
    items.push({
      label: isFav ? 'Remove from Favorites' : 'Add to Favorites',
      icon: isFav ? HeartOff : Heart,
      action: async () => { await favoritesStore.toggle(track.id) },
    })

    const playlistChildren: ContextMenuItem[] = library.playlists.length
      ? library.playlists.map(p => ({
          label: p.name,
          icon: ListPlus,
          action: () => { api.addToPlaylist(p.id, track.id) },
        }))
      : [{ label: 'No playlists', disabled: true }]

    items.push({
      label: 'Add to Playlist',
      icon: ListPlus,
      children: playlistChildren,
    })

    api.getPlaylistsForTrack(track.id).then(playlistIds => {
      if (!playlistIds || !playlistIds.length) return
      playlistChildren.forEach((child, index) => {
        const p = library.playlists[index]
        if (p && playlistIds.includes(p.id)) {
          child.iconRight = Check
          child.action = () => { api.removeFromPlaylist(p.id, track.id) }
        }
      })
    })

    items.push({ separator: true })

    items.push({
      label: 'Go to Album',
      icon: Disc,
      disabled: !track.album,
      action: () => { options.onNavigateAlbum?.(track.album || '') },
    })

    items.push({
      label: 'Go to Artist',
      icon: User,
      disabled: !track.artist,
      action: () => { options.onNavigateArtist?.(track.artist || '') },
    })

    items.push({ separator: true })

    items.push({
      label: 'Edit Metadata',
      icon: Pencil,
      action: () => {
        editingTrack.value = track
        metadataOpen.value = true
      },
    })

    items.push({
      label: 'Show in File Explorer',
      icon: FolderOpen,
      action: () => { api.showInExplorer(track.id) },
    })

    if (options.playlistId && options.playlistId !== 'favorites') {
      items.push({ separator: true })
      items.push({
        label: 'Remove from Playlist',
        icon: Trash2,
        danger: true,
        action: () => { api.removeFromPlaylist(options.playlistId!, track.id) },
      })
    }

    return items
  }

  function buildMultiSelectMenuItems(tracks: Track[], options: TrackContextMenuOptions = {}): ContextMenuItem[] {
    const items: ContextMenuItem[] = []

    items.push({
      label: 'Play Next',
      icon: CornerUpRight,
      action: () => {
        const q = [...playerStore.queue]
        q.splice(playerStore.queueIndex + 1, 0, ...tracks)
        playerStore.queue = q
      },
    })

    items.push({ separator: true })

    items.push({
      label: 'Add to Favorites',
      icon: Heart,
      action: async () => {
        for (const track of tracks) {
          if (!favoritesStore.isFavorite(track.id)) {
            await favoritesStore.toggle(track.id)
          }
        }
      },
    })

    const playlistChildren: ContextMenuItem[] = library.playlists.length
      ? library.playlists.map(p => ({
          label: p.name,
          icon: ListPlus,
          action: () => { tracks.forEach(t => api.addToPlaylist(p.id, t.id)) },
        }))
      : [{ label: 'No playlists', disabled: true }]

    items.push({
      label: 'Add to Playlist',
      icon: ListPlus,
      children: playlistChildren,
    })

    if (options.playlistId && options.playlistId !== 'favorites') {
      items.push({ separator: true })
      items.push({
        label: 'Remove from Playlist',
        icon: Trash2,
        danger: true,
        action: () => {
          tracks.forEach(t => api.removeFromPlaylist(options.playlistId!, t.id))
        },
      })
    }

    return items
  }

  return { buildMenuItems, buildMultiSelectMenuItems, editingTrack, metadataOpen, findLyricsOpen }
}
