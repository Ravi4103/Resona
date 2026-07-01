import { onMounted, onUnmounted, type Ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import type { Track } from '../lib/invoke'
import { useLibraryStore } from '../stores/library'

export function useLibraryUpdates(localTracks: Ref<Track[]>) {
  const library = useLibraryStore()
  let unlistenFns: (() => void)[] = []

  onMounted(async () => {
    unlistenFns = [
      await listen<string>('library:track-updated', (event) => {
        const id = event.payload
        const idx = localTracks.value.findIndex(t => t.id === id)
        if (idx >= 0) {
          const updated = library.tracks.find(t => t.id === id)
          if (updated) localTracks.value[idx] = updated
        }
      }),
      await listen<string[]>('library:track-deleted', (event) => {
        const ids = event.payload
        localTracks.value = localTracks.value.filter(t => !ids.includes(t.id))
      }),
    ]
  })

  onUnmounted(() => {
    unlistenFns.forEach(fn => fn())
  })
}
