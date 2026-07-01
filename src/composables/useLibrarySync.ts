import { onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useLibraryStore } from '../stores/library'
import { useAppStore } from '../stores/app'
import * as api from '../lib/invoke'

export function useLibrarySync() {
  const library = useLibraryStore()
  const appStore = useAppStore()
  let unlistenFns: (() => void)[] = []

  onMounted(async () => {
    unlistenFns = [
      await listen('library:sync-finished', async () => {
        await library.loadAll()
        if (appStore.autoFetchLyrics) {
          api.batchFetchLyrics()
        }
      }),
    ]
  })

  onUnmounted(() => {
    unlistenFns.forEach(fn => fn())
  })
}
