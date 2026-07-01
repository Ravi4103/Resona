import { Play, Shuffle, ListPlus } from 'lucide-vue-next'
import { usePlaylistsStore } from '../stores/playlists'
import type { ContextMenuItem } from './useContextMenu'
import type { Track } from '../lib/invoke'
import * as api from '../lib/invoke'

export function useGroupContextMenu() {
  const playlistsStore = usePlaylistsStore()

  function buildMenuItems(tracks: Track[], playAll: () => void, shuffleAll: () => void): ContextMenuItem[] {
    return [
      { label: 'Play All', icon: Play, action: () => playAll() },
      { label: 'Shuffle', icon: Shuffle, action: () => shuffleAll() },
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
    ]
  }

  return { buildMenuItems }
}
