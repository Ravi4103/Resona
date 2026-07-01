<script setup lang="ts">
import { ref, watch } from 'vue'
import { Plus, Check } from 'lucide-vue-next'
import type { Playlist } from '../lib/invoke'

const props = defineProps<{
  open: boolean
  trackCount: number
  playlists: Playlist[]
}>()

const emit = defineEmits<{
  confirm: [playlistId: string]
  createPlaylist: [name: string]
  close: []
}>()

const selectedId = ref('')
const showCreateInput = ref(false)
const newName = ref('')

watch(() => props.open, (val) => {
  if (val) {
    selectedId.value = ''
    showCreateInput.value = false
    newName.value = ''
  }
})

function select(id: string) {
  selectedId.value = id
}

function confirm() {
  if (selectedId.value) {
    emit('confirm', selectedId.value)
  }
}

function create() {
  const name = newName.value.trim()
  if (name) {
    emit('createPlaylist', name)
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="fixed inset-0 z-[999] flex items-center justify-center" @click.self="emit('close')">
      <div class="absolute inset-0 bg-background/60 backdrop-blur-sm" />
      <div class="relative bg-background border border-foreground/[0.08] rounded-2xl shadow-2xl w-full max-w-sm p-6 space-y-5 max-h-[80vh] flex flex-col">
        <div>
          <h3 class="text-lg font-bold">Add to Playlist</h3>
          <p class="text-sm text-foreground opacity-60 mt-1">{{ trackCount }} track{{ trackCount === 1 ? '' : 's' }}</p>
        </div>

        <div class="flex-1 overflow-y-auto custom-scrollbar -mx-2 px-2 space-y-1">
          <div v-for="pl in playlists" :key="pl.id"
            class="flex items-center gap-3 p-2.5 rounded-xl cursor-pointer transition-colors"
            :class="selectedId === pl.id ? 'bg-primary/10 ring-1 ring-primary/30' : 'hover:bg-foreground/[0.04]'"
            @click="select(pl.id)">
            <div class="w-5 h-5 rounded-md border-2 flex items-center justify-center flex-shrink-0"
              :class="selectedId === pl.id ? 'border-primary bg-primary' : 'border-foreground/30'">
              <Check v-if="selectedId === pl.id" class="w-3.5 h-3.5 text-primary-foreground" />
            </div>
            <span class="text-sm font-medium truncate">{{ pl.name }}</span>
          </div>

          <div v-if="playlists.length === 0 && !showCreateInput"
            class="py-6 text-center text-sm text-foreground opacity-50">
            No playlists yet
          </div>
        </div>

        <div v-if="!showCreateInput">
          <button @click="showCreateInput = true"
            class="w-full flex items-center gap-2 px-4 py-2.5 rounded-xl border-2 border-dashed border-foreground/15 text-foreground opacity-70 hover:opacity-100 hover:border-primary/50 transition-all text-sm font-medium">
            <Plus class="w-4 h-4" />
            New Playlist
          </button>
        </div>

        <div v-else class="flex gap-2">
          <input v-model="newName" ref="nameInput" type="text" placeholder="Playlist name"
            class="flex-1 px-3 py-2 text-sm rounded-xl bg-foreground/[0.06] border border-foreground/15 text-foreground placeholder:text-foreground/40 outline-none focus:border-primary/50 transition-colors"
            @keydown.enter="create" />
          <button @click="create" :disabled="!newName.trim()"
            class="px-4 py-2 rounded-xl bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 transition-opacity disabled:opacity-50 shadow-lg shadow-primary/20">
            Create
          </button>
        </div>

        <div class="flex gap-3 pt-1">
          <button @click="emit('close')"
            class="flex-1 px-4 py-2.5 rounded-xl bg-foreground/[0.06] text-foreground font-medium hover:bg-foreground/[0.1] transition-colors text-sm">
            Cancel
          </button>
          <button @click="confirm" :disabled="!selectedId"
            class="flex-1 px-4 py-2.5 rounded-xl bg-primary text-primary-foreground font-medium hover:opacity-90 transition-opacity text-sm disabled:opacity-50 shadow-lg shadow-primary/20">
            Add
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
