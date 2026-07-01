<script setup lang="ts">
import { ref, watch } from 'vue'
import { X, Save } from 'lucide-vue-next'
import * as api from '../lib/invoke'
import type { Track } from '../lib/invoke'
import { useLibraryStore } from '../stores/library'

const props = defineProps<{
  open: boolean
  track: Track | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const library = useLibraryStore()

const title = ref('')
const artist = ref('')
const album = ref('')
const albumArtist = ref('')
const genre = ref('')
const year = ref(0)
const trackNumber = ref(0)
const discNumber = ref(0)
const saving = ref(false)
const saved = ref(false)

watch(() => props.open, (val) => {
  if (val && props.track) {
    title.value = props.track.title
    artist.value = props.track.artist
    album.value = props.track.album
    albumArtist.value = props.track.album_artist
    genre.value = props.track.raw_genre_names || props.track.genre
    year.value = props.track.year
    trackNumber.value = props.track.track_number
    discNumber.value = props.track.disc_number
    saved.value = false
    saving.value = false
  }
})

async function save() {
  if (!props.track) return
  saving.value = true
  try {
    await api.updateTrackMetadata({
      id: props.track.id,
      title: title.value,
      artist: artist.value,
      album: album.value,
      album_artist: albumArtist.value,
      genre: genre.value,
      year: year.value,
      track_number: trackNumber.value,
      disc_number: discNumber.value,
    })
    await library.loadAll()
    saved.value = true
    setTimeout(() => emit('update:open', false), 1000)
  } catch (e: any) {
    console.error('Failed to save metadata', e)
  }
  saving.value = false
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="fixed inset-0 z-[9999] flex items-center justify-center bg-background/60 backdrop-blur-sm"
      @click.self="emit('update:open', false)">
      <div class="w-[420px] bg-glass-elevated backdrop-blur-2xl ring-1 ring-border-glass rounded-2xl shadow-2xl flex flex-col overflow-hidden">
        <div class="flex items-center justify-between px-5 py-4 border-b border-foreground/[0.06]">
          <h2 class="text-sm font-semibold">Edit Metadata</h2>
          <button @click="emit('update:open', false)" class="p-1 rounded-full hover:bg-foreground/10 transition-colors">
            <X class="w-4 h-4" />
          </button>
        </div>

        <div class="p-5 space-y-4 overflow-y-auto">
          <div v-if="saved" class="flex items-center justify-center py-8 text-green-400 text-sm font-medium">
            <Save class="w-4 h-4 mr-2" /> Saved!
          </div>

          <template v-else>
            <div v-for="field in ([
              { key: 'title', label: 'Title', model: title },
              { key: 'artist', label: 'Artist', model: artist },
              { key: 'album', label: 'Album', model: album },
              { key: 'albumArtist', label: 'Album Artist', model: albumArtist },
              { key: 'genre', label: 'Genre', model: genre },
            ] as const)" :key="field.key" class="space-y-1">
              <label class="text-xs font-medium text-foreground/70">{{ field.label }}</label>
              <input v-model="field.model" type="text"
                class="w-full px-3 py-2 text-sm rounded-xl bg-foreground/[0.06] border border-foreground/15 text-foreground outline-none focus:border-primary/50 transition-colors" />
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div class="space-y-1">
                <label class="text-xs font-medium text-foreground/70">Year</label>
                <input v-model="year" type="number"
                  class="w-full px-3 py-2 text-sm rounded-xl bg-foreground/[0.06] border border-foreground/15 text-foreground outline-none focus:border-primary/50 transition-colors" />
              </div>
              <div class="space-y-1">
                <label class="text-xs font-medium text-foreground/70">Track #</label>
                <input v-model="trackNumber" type="number"
                  class="w-full px-3 py-2 text-sm rounded-xl bg-foreground/[0.06] border border-foreground/15 text-foreground outline-none focus:border-primary/50 transition-colors" />
              </div>
            </div>

            <button @click="save" :disabled="saving"
              class="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-primary text-primary-foreground rounded-xl font-medium hover:opacity-90 transition-all disabled:opacity-50 text-sm">
              <Save class="w-4 h-4" />
              {{ saving ? 'Saving...' : 'Save' }}
            </button>
          </template>
        </div>
      </div>
    </div>
  </Teleport>
</template>
