<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, reactive } from 'vue'
import { RotateCcw, Plus, Trash2, Folder, FolderOpen, RefreshCw, FileText, Loader2 } from 'lucide-vue-next'
import { useLibraryStore } from '../../stores/library'
import { useAppStore } from '../../stores/app'
import { listen } from '@tauri-apps/api/event'
import * as api from '../../lib/invoke'
import ConfirmDialog from '../ConfirmDialog.vue'

const library = useLibraryStore()
const appStore = useAppStore()

const folders = ref<string[]>([])
const scanning = ref(false)
const scanResult = ref<string | null>(null)
const folderToDelete = ref<string | null>(null)
const isLoading = ref(true)

const batchFetching = ref(false)
const fetchingAll = ref(false)
const activeFolder = ref<string | null>(null)
const batchCurrent = ref(0)
const batchTotal = ref(0)
const folderResults = reactive<Record<string, string>>({})
const addingFolders = ref(false)
const folderScanStatus = reactive<Record<string, string>>({})
let unlistenProgress: (() => void) | null = null
let unlistenFolderDone: (() => void) | null = null
let unlistenAllDone: (() => void) | null = null
let unlistenFolderScan: (() => void) | null = null

const batchProgress = computed(() =>
  batchTotal.value > 0 ? Math.round((batchCurrent.value / batchTotal.value) * 100) : 0
)

onMounted(async () => {
  try {
    unlistenProgress = await listen<{ current: number; total: number; track: string; status: string; folder: string }>('batch-lyrics-progress', (event) => {
      const p = event.payload
      batchCurrent.value = p.current
      batchTotal.value = p.total
      activeFolder.value = p.folder
    })
    unlistenFolderDone = await listen<{ folder: string; fetched: number; skipped_already_have: number; skipped_not_found: number; total: number }>('batch-lyrics-folder-done', (event) => {
      const p = event.payload
      if (p.fetched > 0) {
        let msg = `Fetched ${p.fetched} lyrics`
        if (p.skipped_not_found > 0 || p.skipped_already_have > 0) {
          msg += ` (${p.skipped_already_have} had, ${p.skipped_not_found} not found)`
        }
        folderResults[p.folder] = msg
      } else if (p.skipped_already_have === p.total) {
        folderResults[p.folder] = 'All lyrics already fetched'
      } else if (p.skipped_not_found > 0 && p.skipped_already_have > 0) {
        folderResults[p.folder] = `No lyrics found for ${p.skipped_not_found} tracks (${p.skipped_already_have} already have)`
      } else if (p.skipped_not_found > 0) {
        folderResults[p.folder] = `No lyrics found for ${p.skipped_not_found} tracks`
      } else {
        folderResults[p.folder] = 'No new lyrics found'
      }
    })
    unlistenAllDone = await listen('batch-lyrics-all-done', () => {
      batchFetching.value = false
      fetchingAll.value = false
      activeFolder.value = null
      batchCurrent.value = 0
      batchTotal.value = 0
    })
    unlistenFolderScan = await listen<{ path: string; status: string; added?: number; error?: string }>('folder-scan-progress', (event) => {
      const p = event.payload
      if (p.status === 'scanning') {
        folderScanStatus[p.path] = 'scanning'
      } else if (p.status === 'done') {
        folderScanStatus[p.path] = `done:${p.added ?? 0}`
      } else if (p.status === 'error') {
        folderScanStatus[p.path] = `error:${p.error ?? 'unknown'}`
      }
    })
  } catch {}
})

onUnmounted(() => {
  unlistenProgress?.()
  unlistenFolderDone?.()
  unlistenAllDone?.()
  unlistenFolderScan?.()
})

function folderStatusClass(result: string): string {
  if (!result || result === 'pending' || result === 'Fetching...') return 'text-foreground/60'
  if (result === 'already_running' || result.startsWith('Error')) return 'text-error'
  if (result.startsWith('Fetched') || result === 'All lyrics already fetched') return 'text-green-400'
  if (result.startsWith('No lyrics')) return 'text-amber-400'
  return 'text-foreground/60'
}

async function fetchForFolder(folder: string) {
  if (batchFetching.value) {
    folderResults[folder] = 'already_running'
    return
  }
  batchFetching.value = true
  fetchingAll.value = false
  activeFolder.value = folder
  batchCurrent.value = 0
  batchTotal.value = 0
  folderResults[folder] = 'Fetching...'
  try {
    const result = await api.batchFetchLyrics(folder)
    if (result === 'already_running') {
      folderResults[folder] = 'already_running'
      batchFetching.value = false
      activeFolder.value = null
    } else if (result === 'nothing_to_do') {
      folderResults[folder] = 'No tracks in this folder'
      batchFetching.value = false
      activeFolder.value = null
    } else if (result === 'all_done') {
      folderResults[folder] = 'All lyrics already fetched'
      batchFetching.value = false
      activeFolder.value = null
    }
    // "started" — background task handles completion events
  } catch (e: any) {
    folderResults[folder] = `Error: ${e}`
    batchFetching.value = false
    activeFolder.value = null
  }
}

async function fetchAll() {
  if (batchFetching.value) return
  batchFetching.value = true
  fetchingAll.value = true
  activeFolder.value = null
  batchCurrent.value = 0
  batchTotal.value = 0
  for (const f of folders.value) folderResults[f] = 'pending'
  try {
    const result = await api.batchFetchLyrics()
    if (result === 'already_running' || result === 'nothing_to_do' || result === 'all_done') {
      for (const f of folders.value) folderResults[f] = result === 'all_done' ? 'All lyrics already fetched' : result
      batchFetching.value = false
      fetchingAll.value = false
    }
    // "started" — events handle per-folder results and final reset
  } catch (e: any) {
    const current = activeFolder.value
    if (current) folderResults[current] = `Error: ${e}`
    batchFetching.value = false
    fetchingAll.value = false
  }
}

async function loadFolders(showLoader = true) {
  if (showLoader) isLoading.value = true
  try {
    folders.value = await api.getLibraryFolders()
    for (const f of folders.value) {
      if (!(f in folderResults)) folderResults[f] = ''
    }
    // Clean up folderScanStatus for paths no longer in the list
    for (const key of Object.keys(folderScanStatus)) {
      if (!folders.value.includes(key)) {
        delete folderScanStatus[key]
      }
    }
  } catch {}
  if (showLoader) isLoading.value = false
}

onMounted(() => loadFolders())

async function pickAndAddFolder() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({ directory: true, multiple: true })
    if (!selected) return
    const paths = Array.isArray(selected) ? selected : [selected]
    addingFolders.value = true
    for (const p of paths) folderScanStatus[p] = 'waiting'
    const result = await api.addLibraryFolders(paths)
    addingFolders.value = false
    scanResult.value = `Added: ${result.added}, Removed: ${result.removed}`
    await loadFolders(false)
    await library.loadAll()
  } catch (e: any) {
    addingFolders.value = false
    scanResult.value = `Error: ${e}`
  }
}

async function removeFolder() {
  if (!folderToDelete.value) return
  const path = folderToDelete.value
  folderToDelete.value = null
  delete folderResults[path]
  try {
    await api.removeLibraryFolder(path)
    await library.loadAll()
    await loadFolders(false)
  } catch {}
}

async function syncAllFolders() {
  scanning.value = true
  scanResult.value = null
  let totalAdded = 0
  try {
    for (const folder of folders.value) {
      const result = await api.scanFolder(folder)
      totalAdded += result.added
    }
    scanResult.value = `Library synced. ${totalAdded} tracks added/updated.`
    await library.loadAll()
  } catch (e: any) { scanResult.value = `Error: ${e}` }
  scanning.value = false
  if (appStore.autoFetchLyrics) {
    fetchAll()
  }
}

async function scanFolder(path: string) {
  scanning.value = true
  try {
    const result = await api.scanFolder(path)
    scanResult.value = `Added: ${result.added}, Updated: ${result.updated}, Removed: ${result.removed}`
    await library.loadAll()
  } catch (e: any) { scanResult.value = `Error: ${e}` }
  scanning.value = false
  if (appStore.autoFetchLyrics) {
    fetchForFolder(path)
  }
}

</script>

<template>
  <div class="space-y-8 animate-in fade-in slide-in-from-bottom-2 duration-500">
    <div class="flex items-center justify-between mb-4 select-none">
      <h2 class="text-xl font-bold">Library</h2>
      <div class="flex gap-3">
        <button @click="syncAllFolders" :disabled="scanning || folders.length === 0"
          class="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-xl hover:opacity-90 transition-all disabled:opacity-50 text-sm font-bold shadow-lg shadow-primary/20">
          <RotateCcw class="w-4 h-4" :class="{ 'animate-spin': scanning }" />
          {{ scanning ? 'Syncing...' : 'Sync Library' }}
        </button>
      </div>
    </div>

    <section class="bg-card rounded-2xl border border-foreground/[0.06] p-6">
      <div class="flex items-center justify-between mb-6">
        <div>
          <h3 class="text-lg font-bold mb-1">Music Folders</h3>
          <p class="text-sm text-foreground opacity-60">Manage your music library folders</p>
        </div>
        <button @click="pickAndAddFolder" :disabled="scanning || addingFolders"
          class="flex items-center gap-2 px-4 py-2 bg-foreground/[0.04] text-foreground rounded-xl hover:bg-foreground/[0.08] transition-all text-sm font-bold disabled:opacity-50">
          <Loader2 v-if="addingFolders" class="w-4 h-4 animate-spin" />
          <Plus v-else class="w-4 h-4" />
          {{ addingFolders ? 'Adding...' : 'Add Folder' }}
        </button>
      </div>

      <div v-if="isLoading" class="py-12 flex justify-center">
        <RotateCcw class="w-8 h-8 animate-spin text-foreground opacity-40" />
      </div>

      <div v-else-if="folders.length === 0"
        class="py-12 text-center border-2 border-dashed border-foreground/[0.06] rounded-2xl">
        <Folder class="w-12 h-12 mx-auto text-foreground opacity-30 mb-4" />
        <p class="text-foreground opacity-60 text-sm font-medium">No folders added yet</p>
      </div>

      <ul v-else class="space-y-2">
        <li v-for="folder in folders" :key="folder"
          class="flex items-center justify-between p-4 bg-foreground/[0.02] border border-foreground/[0.04] rounded-xl group transition-all hover:bg-foreground/[0.04]">
          <div class="flex items-center gap-4 overflow-hidden min-w-0">
            <div class="p-2 bg-background rounded-lg shadow-sm">
              <Loader2 v-if="folderScanStatus[folder] === 'scanning'" class="w-4 h-4 animate-spin text-primary" />
              <FolderOpen v-else class="w-4 h-4 text-foreground opacity-60" />
            </div>
            <div class="min-w-0">
              <span class="text-sm font-bold truncate" :title="folder">{{ folder }}</span>
              <p v-if="folderScanStatus[folder]" class="text-xs mt-0.5"
                :class="folderScanStatus[folder] === 'scanning' ? 'text-primary' : folderScanStatus[folder]?.startsWith('done') ? 'text-success' : folderScanStatus[folder]?.startsWith('error') ? 'text-error' : 'text-foreground/60'">
                <template v-if="folderScanStatus[folder] === 'waiting'">Waiting...</template>
                <template v-else-if="folderScanStatus[folder] === 'scanning'">Scanning...</template>
                <template v-else-if="folderScanStatus[folder]?.startsWith('done')">{{ folderScanStatus[folder]?.replace('done:', '') }} tracks added</template>
                <template v-else-if="folderScanStatus[folder]?.startsWith('error')">{{ folderScanStatus[folder]?.replace('error:', 'Error: ') }}</template>
              </p>
            </div>
          </div>
          <div class="flex items-center gap-1">
            <button @click="scanFolder(folder)" :disabled="scanning || addingFolders"
              class="p-2 text-foreground/50 hover:text-foreground rounded-lg transition-colors disabled:opacity-40" title="Refresh">
              <RefreshCw class="w-3.5 h-3.5" :class="{ 'animate-spin': scanning }" />
            </button>
            <button @click="folderToDelete = folder" :disabled="scanning || addingFolders"
              class="p-2 text-error hover:text-error hover:bg-error/10 rounded-lg transition-all disabled:opacity-50">
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          </div>
        </li>
      </ul>

      <p v-if="scanResult" class="mt-4 text-sm" :class="scanResult.startsWith('Error') ? 'text-error' : 'text-success'">{{ scanResult }}</p>
      <p class="mt-2 text-sm text-foreground/50">{{ library.tracks.length }} tracks, {{ library.playlists.length }} playlists</p>
    </section>

    <section class="bg-card rounded-2xl border border-foreground/[0.06] p-6">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h3 class="text-lg font-bold mb-1">Lyrics Fetching</h3>
          <p class="text-sm text-foreground opacity-60">Fetch missing lyrics from online sources for each folder</p>
        </div>
        <button @click="fetchAll" :disabled="batchFetching || folders.length === 0"
          class="flex items-center gap-2 px-4 py-2 rounded-xl transition-all text-sm font-bold disabled:opacity-50"
          :class="batchFetching && fetchingAll ? 'bg-primary/20 text-primary' : 'bg-primary text-primary-foreground hover:opacity-90'">
          <Loader2 v-if="batchFetching && fetchingAll" class="w-4 h-4 animate-spin" />
          <FileText v-else class="w-4 h-4" />
          {{ batchFetching && fetchingAll ? 'Fetching...' : 'Fetch All' }}
        </button>
      </div>

      <!-- Global progress bar -->
      <div v-if="batchFetching" class="mb-4 space-y-2">
        <div class="w-full h-2 bg-foreground/[0.06] rounded-full overflow-hidden">
          <div class="h-full bg-primary rounded-full transition-all duration-300 ease-out"
            :style="{ width: batchProgress + '%' }" />
        </div>
        <p class="text-xs text-foreground/60">
          <template v-if="activeFolder">{{ activeFolder }} — </template>
          {{ batchCurrent }} / {{ batchTotal }} tracks
        </p>
      </div>

      <div v-if="folders.length === 0"
        class="py-8 text-center border-2 border-dashed border-foreground/[0.06] rounded-2xl">
        <FileText class="w-10 h-10 mx-auto text-foreground opacity-30 mb-3" />
        <p class="text-foreground opacity-60 text-sm">Add a music folder to start fetching lyrics</p>
      </div>

      <ul v-else class="space-y-2">
        <li v-for="folder in folders" :key="folder"
          class="flex items-center justify-between p-4 bg-foreground/[0.02] border border-foreground/[0.04] rounded-xl group transition-all hover:bg-foreground/[0.04]">
          <div class="flex items-center gap-4 overflow-hidden min-w-0">
            <div class="p-2 bg-background rounded-lg shadow-sm">
              <FolderOpen class="w-4 h-4 text-foreground opacity-60" />
            </div>
            <div class="min-w-0">
              <p class="text-sm font-bold truncate" :title="folder">{{ folder }}</p>
              <p v-if="folderResults[folder]" class="text-xs mt-0.5"
                :class="folderStatusClass(folderResults[folder])">
                {{ folderResults[folder] === 'pending' ? 'Waiting...' : folderResults[folder] }}
              </p>
            </div>
          </div>
          <div class="flex items-center gap-1 flex-shrink-0">
            <button @click="fetchForFolder(folder)" :disabled="batchFetching"
              class="p-2 text-foreground/50 hover:text-foreground rounded-lg transition-colors disabled:opacity-40"
              :title="batchFetching && activeFolder === folder ? 'Fetching...' : 'Fetch Lyrics'">
              <Loader2 v-if="batchFetching && activeFolder === folder" class="w-3.5 h-3.5 animate-spin" />
              <RefreshCw v-else class="w-3.5 h-3.5" />
            </button>
          </div>
        </li>
      </ul>
    </section>

    <ConfirmDialog
      :open="!!folderToDelete"
      @cancel="folderToDelete = null"
      title="Remove Folder"
      message="Remove this folder from your library? Tracks won't be deleted from disk."
      confirm-label="Remove"
      danger
      @confirm="removeFolder"
    />
  </div>
</template>
