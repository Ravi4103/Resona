<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Radio, Play, Square, Plus, Trash2, Globe, Wifi } from 'lucide-vue-next'
import * as api from '../lib/invoke'
import ConfirmDialog from '../components/ConfirmDialog.vue'

const stations = ref<api.RadioStation[]>([])
const isPlaying = ref(false)
const currentUrl = ref('')
const showAdd = ref(false)
const showDeleteConfirm = ref(false)
const pendingDeleteId = ref('')
const pendingDeleteName = ref('')

const newStation = ref({
  id: '',
  name: '',
  url: '',
  genre: '',
  logo_url: '',
  country: '',
  language: '',
  bitrate: 0,
})

function generateId() {
  return crypto.randomUUID ? crypto.randomUUID() : Date.now().toString(36) + Math.random().toString(36).slice(2)
}

onMounted(async () => {
  await loadStations()
})

function onRadioEvent(e: CustomEvent) {
  if (e.detail === false) {
    isPlaying.value = false
    currentUrl.value = ''
  }
}

onMounted(() => {
  window.addEventListener('radio:playing', onRadioEvent as EventListener)
  window.addEventListener('radio:ended', (() => {
    isPlaying.value = false
    currentUrl.value = ''
  }) as EventListener)
})

onUnmounted(() => {
  window.removeEventListener('radio:playing', onRadioEvent as EventListener)
})

async function loadStations() {
  try {
    stations.value = await api.getRadioStations()
    const status = await api.radioStatus()
    isPlaying.value = status
  } catch (e) {
    console.error('Failed to load radio stations', e)
  }
}

async function togglePlay(station: api.RadioStation) {
  if (isPlaying.value && currentUrl.value === station.url) {
    try {
      await api.stopRadio()
      isPlaying.value = false
      currentUrl.value = ''
    } catch (e) {
      console.error('Failed to stop radio', e)
    }
  } else {
    try {
      await api.playRadio(station.url)
      isPlaying.value = true
      currentUrl.value = station.url
    } catch (e) {
      console.error('Failed to play radio', e)
    }
  }
}

async function addStation() {
  const s = newStation.value
  if (!s.name || !s.url) return
  s.id = generateId()
  try {
    await api.addRadioStation({ ...s })
    await loadStations()
    showAdd.value = false
    newStation.value = { id: '', name: '', url: '', genre: '', logo_url: '', country: '', language: '', bitrate: 0 }
  } catch (e) {
    console.error('Failed to add station', e)
  }
}

function confirmDelete(station: api.RadioStation) {
  pendingDeleteId.value = station.id
  pendingDeleteName.value = station.name
  showDeleteConfirm.value = true
}

async function deleteStation() {
  try {
    await api.deleteRadioStation(pendingDeleteId.value)
    if (currentUrl.value && stations.value.find(s => s.id === pendingDeleteId.value)?.url === currentUrl.value) {
      await api.stopRadio()
      isPlaying.value = false
      currentUrl.value = ''
    }
    await loadStations()
  } catch (e) {
    console.error('Failed to delete station', e)
  }
  showDeleteConfirm.value = false
}
</script>

<template>
  <div class="p-6 h-full overflow-y-auto">
    <div class="flex items-center justify-between mb-6">
      <div class="flex items-center gap-3">
        <Radio class="w-6 h-6 text-primary" />
        <h1 class="text-2xl font-bold">Internet Radio</h1>
      </div>
      <button
        class="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:opacity-90 transition-opacity"
        @click="showAdd = true"
      >
        <Plus class="w-4 h-4" />
        Add Station
      </button>
    </div>

    <div v-if="isPlaying" class="mb-4 px-4 py-3 bg-primary/10 rounded-lg flex items-center gap-3 text-sm">
      <Wifi class="w-4 h-4 text-primary animate-pulse" />
      <span class="text-primary font-medium">Now Playing</span>
      <span class="text-muted-foreground">{{ stations.find(s => s.url === currentUrl)?.name || 'Radio' }}</span>
    </div>

    <!-- Add Station Dialog -->
    <div v-if="showAdd" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showAdd = false">
      <div class="bg-card rounded-xl p-6 w-full max-w-md mx-4 shadow-2xl border border-border">
        <h2 class="text-lg font-semibold mb-4">Add Radio Station</h2>
        <div class="space-y-3">
          <input v-model="newStation.name" placeholder="Station Name" class="w-full px-3 py-2 rounded-lg bg-background border border-input focus:outline-none focus:ring-2 focus:ring-primary" />
          <input v-model="newStation.url" placeholder="Stream URL (e.g. http://example.com/stream.mp3)" class="w-full px-3 py-2 rounded-lg bg-background border border-input focus:outline-none focus:ring-2 focus:ring-primary" />
          <div class="flex gap-3">
            <input v-model="newStation.genre" placeholder="Genre" class="flex-1 px-3 py-2 rounded-lg bg-background border border-input focus:outline-none focus:ring-2 focus:ring-primary" />
            <input v-model="newStation.bitrate" type="number" placeholder="Bitrate" class="w-24 px-3 py-2 rounded-lg bg-background border border-input focus:outline-none focus:ring-2 focus:ring-primary" />
          </div>
          <div class="flex gap-3">
            <input v-model="newStation.country" placeholder="Country" class="flex-1 px-3 py-2 rounded-lg bg-background border border-input focus:outline-none focus:ring-2 focus:ring-primary" />
            <input v-model="newStation.language" placeholder="Language" class="flex-1 px-3 py-2 rounded-lg bg-background border border-input focus:outline-none focus:ring-2 focus:ring-primary" />
          </div>
        </div>
        <div class="flex gap-3 mt-6">
          <button class="flex-1 px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:opacity-90" @click="addStation">Add</button>
          <button class="flex-1 px-4 py-2 bg-muted text-muted-foreground rounded-lg hover:opacity-90" @click="showAdd = false">Cancel</button>
        </div>
      </div>
    </div>

    <ConfirmDialog
      :open="showDeleteConfirm"
      title="Delete Station"
      :message="`Remove '${pendingDeleteName}' from your station list?`"
      confirm-label="Delete"
      @confirm="deleteStation"
      @cancel="showDeleteConfirm = false"
    />

    <div v-if="stations.length === 0" class="text-center py-20 text-muted-foreground">
      <Radio class="w-12 h-12 mx-auto mb-4 opacity-30" />
      <p class="text-lg">No radio stations added yet</p>
      <p class="text-sm mt-1">Click "Add Station" to add your first internet radio station</p>
    </div>

    <div v-else class="grid gap-2">
      <div
        v-for="station in stations"
        :key="station.id"
        :class="[
          'flex items-center gap-4 px-4 py-3 rounded-lg transition-colors group',
          currentUrl === station.url && isPlaying
            ? 'bg-primary/10 border border-primary/20'
            : 'bg-card hover:bg-accent/50 border border-transparent'
        ]"
      >
        <button
          class="flex-shrink-0 w-10 h-10 rounded-full flex items-center justify-center transition-colors"
          :class="currentUrl === station.url && isPlaying ? 'bg-primary text-primary-foreground' : 'bg-primary/10 text-primary hover:bg-primary hover:text-primary-foreground'"
          @click="togglePlay(station)"
        >
          <Play v-if="!(currentUrl === station.url && isPlaying)" class="w-4 h-4 ml-0.5" />
          <Square v-else class="w-3.5 h-3.5" />
        </button>

        <div class="flex-1 min-w-0">
          <div class="font-medium truncate">{{ station.name }}</div>
          <div class="flex items-center gap-3 text-xs text-muted-foreground mt-0.5">
            <span v-if="station.genre" class="flex items-center gap-1">
              <Globe class="w-3 h-3" />{{ station.genre }}
            </span>
            <span v-if="station.bitrate > 0">{{ station.bitrate }} kbps</span>
            <span v-if="station.country">{{ station.country }}</span>
            <span v-if="station.language">{{ station.language }}</span>
          </div>
        </div>

        <button
          class="opacity-0 group-hover:opacity-100 p-2 text-muted-foreground hover:text-destructive transition-all"
          @click="confirmDelete(station)"
        >
          <Trash2 class="w-4 h-4" />
        </button>
      </div>
    </div>
  </div>
</template>
