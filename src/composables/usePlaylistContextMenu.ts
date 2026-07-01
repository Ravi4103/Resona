import { Pencil, Trash2, Download, Play } from 'lucide-vue-next'
import type { Playlist } from '../lib/invoke'
import type { ContextMenuItem } from './useContextMenu'

export function buildPlaylistMenuItems(
  playlist: Playlist,
  options: {
    onPlay?: (p: Playlist) => void
    onRename?: (p: Playlist) => void
    onDelete?: (p: Playlist) => void
    onExport?: (p: Playlist) => void
  },
): ContextMenuItem[] {
  const items: ContextMenuItem[] = []

  if (options.onPlay) {
    items.push({ label: 'Play', icon: Play, action: () => options.onPlay!(playlist) })
    items.push({ separator: true })
  }
  if (options.onRename) {
    items.push({ label: 'Rename', icon: Pencil, action: () => options.onRename!(playlist) })
  }
  if (options.onExport) {
    items.push({ label: 'Export as M3U', icon: Download, action: () => options.onExport!(playlist) })
  }
  if (options.onDelete) {
    if (items.length > 0) items.push({ separator: true })
    items.push({ label: 'Delete', icon: Trash2, danger: true, action: () => options.onDelete!(playlist) })
  }
  return items
}
