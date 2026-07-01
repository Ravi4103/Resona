<script setup lang="ts">
import { ref, watch } from 'vue'
import { Search, X, Loader2, Music, User, Globe } from 'lucide-vue-next'
import { usePlayerStore } from '../stores/player'
import * as api from '../lib/invoke'
import type { Track, LyricsResult } from '../lib/invoke'

const props = defineProps<{
  open: boolean
  track: Track | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const playerStore = usePlayerStore()

const searchTitle = ref('')
const searchArtist = ref('')
const isSearching = ref(false)
const results = ref<LyricsResult[]>([])
const selectedIndex = ref(-1)

watch(() => props.open, (val) => {
  if (val && props.track) {
    searchTitle.value = props.track.title
    searchArtist.value = props.track.artist
    results.value = []
    selectedIndex.value = -1
  }
})

async function search() {
  if (!searchTitle.value || isSearching.value) return
  isSearching.value = true
  results.value = []
  selectedIndex.value = -1
  try {
    const res = await api.findLyrics(searchTitle.value, searchArtist.value, '', undefined)
    results.value = res || []
  } catch (e) {
    console.error('Failed to search lyrics', e)
  } finally {
    isSearching.value = false
  }
}

async function save() {
  if (selectedIndex.value === -1 || !props.track) return
  const selected = results.value[selectedIndex.value]
  try {
    await api.saveLyrics(props.track.id, selected.lyrics, selected.source)
    if (playerStore.currentTrack?.id === props.track.id) {
      playerStore.lyricsContent = selected.lyrics || undefined
    }
    emit('update:open', false)
  } catch (e) {
    console.error('Failed to save lyrics', e)
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="fixed inset-0 z-[9999] flex items-center justify-center bg-background/60 backdrop-blur-sm"
      @click.self="emit('update:open', false)">
      <div class="w-[900px] max-w-[90vw] h-[80vh] max-h-[90vh] bg-glass-elevated backdrop-blur-2xl ring-1 ring-border-glass rounded-2xl shadow-2xl flex flex-col overflow-hidden">
        <!-- Header -->
        <div class="flex items-center justify-between px-5 py-4 border-b border-foreground/[0.06]">
          <h2 class="text-sm font-semibold flex items-center gap-2">
            <Search class="w-4 h-4" />
            Find Lyrics
          </h2>
          <button class="p-1.5 rounded-full hover:bg-foreground/10 transition-colors text-foreground opacity-60 hover:text-foreground"
            @click="emit('update:open', false)">
            <X class="w-4 h-4" />
          </button>
        </div>

        <div class="flex-1 flex overflow-hidden">
          <!-- Left: Search & List -->
          <div class="w-1/2 flex flex-col border-r border-foreground/[0.06]">
            <div class="p-4 space-y-3 border-b border-foreground/[0.06]">
              <div class="space-y-1.5">
                <label class="text-[10px] font-semibold uppercase text-foreground/50">Track Title</label>
                <div class="relative">
                  <Music class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-foreground/40 z-10 pointer-events-none" />
                  <input v-model="searchTitle" type="text"
                    class="w-full pl-9 pr-3 py-2 bg-foreground/[0.04] border border-foreground/[0.08] rounded-lg text-sm outline-none focus:border-primary/50 transition-colors"
                    @keyup.enter="search" />
                </div>
              </div>
              <div class="space-y-1.5">
                <label class="text-[10px] font-semibold uppercase text-foreground/50">Artist</label>
                <div class="relative">
                  <User class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-foreground/40 z-10 pointer-events-none" />
                  <input v-model="searchArtist" type="text"
                    class="w-full pl-9 pr-3 py-2 bg-foreground/[0.04] border border-foreground/[0.08] rounded-lg text-sm outline-none focus:border-primary/50 transition-colors"
                    @keyup.enter="search" />
                </div>
              </div>
              <button
                class="w-full bg-primary hover:bg-primary/90 text-primary-foreground text-sm font-semibold py-2 rounded-lg flex items-center justify-center gap-2 transition-colors disabled:opacity-50"
                :disabled="isSearching || !searchTitle"
                @click="search">
                <Loader2 v-if="isSearching" class="w-4 h-4 animate-spin" />
                <Search v-else class="w-4 h-4" />
                Search
              </button>
            </div>

            <div class="flex-1 overflow-y-auto p-2 space-y-1">
              <div v-if="results.length === 0 && !isSearching" class="h-full flex flex-col items-center justify-center text-foreground/50 p-4 text-center">
                <Search class="w-10 h-10 mb-2 opacity-20" />
                <p class="text-sm">No results found</p>
              </div>

              <div v-for="(res, index) in results" :key="index"
                class="p-3 rounded-lg cursor-pointer transition-colors flex items-center gap-3"
                :class="selectedIndex === index ? 'bg-primary/15 text-primary ring-1 ring-primary/30' : 'hover:bg-foreground/[0.04]'"
                @click="selectedIndex = index">
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium truncate">{{ res.title }}</div>
                  <div class="text-xs text-foreground/60 truncate">{{ res.artist }}</div>
                </div>
                <div class="flex flex-col items-end gap-1">
                  <div class="text-[10px] uppercase font-bold px-1.5 py-0.5 rounded bg-foreground/[0.06] text-foreground/60">
                    {{ res.source }}
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Right: Preview -->
          <div class="w-1/2 flex flex-col bg-background/40">
            <div class="px-4 py-3 border-b border-foreground/[0.06]">
              <h3 class="text-[10px] font-semibold uppercase text-foreground/50 flex items-center gap-2">
                <Globe class="w-3 h-3" />
                Lyrics Preview
              </h3>
            </div>
            <div class="flex-1 overflow-y-auto p-5">
              <template v-if="selectedIndex !== -1">
                <pre class="whitespace-pre-wrap font-sans text-sm text-foreground/80 leading-relaxed">{{ results[selectedIndex].lyrics }}</pre>
              </template>
              <div v-else class="h-full flex flex-col items-center justify-center text-foreground/50">
                <Music class="w-10 h-10 mb-2 opacity-20" />
                <p class="text-sm">Select a result to preview</p>
              </div>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="flex items-center justify-end gap-3 px-5 py-4 border-t border-foreground/[0.06]">
          <button
            class="px-5 py-2 text-sm font-medium rounded-lg hover:bg-foreground/10 transition-colors"
            @click="emit('update:open', false)">
            Cancel
          </button>
          <button
            class="px-5 py-2 text-sm font-semibold rounded-lg bg-primary text-primary-foreground hover:opacity-90 transition-opacity disabled:opacity-40"
            :disabled="selectedIndex === -1"
            @click="save">
            Save Selected
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
