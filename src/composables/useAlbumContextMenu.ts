import { ListEnd, ListPlus, Play, Shuffle } from 'lucide-vue-next'
import { usePlayerStore } from '../stores/player'
import { usePlaylistsStore } from '../stores/playlists'
import type { ContextMenuItem } from './useContextMenu'
import type { Track } from '../lib/invoke'
import * as api from '../lib/invoke'

export function useAlbumContextMenu() {
  const playerStore = usePlayerStore()
  const playlistsStore = usePlaylistsStore()

  function buildMenuItems(tracks: Track[], options?: { hidePlayShuffle?: boolean }): ContextMenuItem[] {
    const items: ContextMenuItem[] = []

    if (!options?.hidePlayShuffle) {
      items.push(
        { label: 'Play', icon: Play, action: () => { playerStore.playTracks(tracks, 0) } },
        { label: 'Shuffle', icon: Shuffle, action: () => { playerStore.setShuffle(true); playerStore.playTracks(tracks, 0) } },
      )
    }

    items.push({
      label: 'Play Next',
      icon: ListEnd,
      action: () => { const q = [...playerStore.queue]; q.splice(playerStore.queueIndex + 1, 0, ...tracks); playerStore.queue = q },
    })

    items.push(
      { separator: true },
      {
        label: 'Add to Playlist',
        icon: ListPlus,
        children: playlistsStore.playlists.length
          ? playlistsStore.playlists.map(p => ({
              label: p.name,
              action: async () => { for (const t of tracks) { await api.addTracksToPlaylist(p.id, [t.id]) } },
            }))
          : [{ label: 'No playlists', disabled: true }],
      },
    )

    return items
  }

  return { buildMenuItems }
}
