<script setup lang="ts">
import { useAppStore } from '../../stores/app'
import { ImagePlay, MicVocal } from 'lucide-vue-next'

const appStore = useAppStore()
</script>

<template>
  <div class="space-y-10 animate-in fade-in slide-in-from-bottom-2 duration-500">
    <section>
      <div class="flex items-center gap-2 mb-6 text-foreground opacity-60 select-none">
        <ImagePlay class="w-4 h-4" />
        <h2 class="text-sm font-bold uppercase tracking-wider">Artwork</h2>
      </div>
      <div class="bg-card rounded-2xl border border-foreground/[0.06] p-5 flex items-center justify-between">
        <div>
          <p class="text-sm font-semibold">Online Artist Artwork</p>
          <p class="text-xs text-foreground opacity-60 mt-1">Fetch missing artist images from online sources</p>
        </div>
        <button @click="appStore.updateUseOnlineArtistArtwork(!appStore.useOnlineArtistArtwork)"
          :class="['w-11 h-6 rounded-full transition-colors relative', appStore.useOnlineArtistArtwork ? 'bg-primary' : 'bg-foreground/20']">
          <span :class="['absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform', appStore.useOnlineArtistArtwork ? 'translate-x-5.5 left-0.5' : 'left-0.5']" />
        </button>
      </div>
    </section>

    <section>
      <div class="flex items-center gap-2 mb-6 text-foreground opacity-60 select-none">
        <MicVocal class="w-4 h-4" />
        <h2 class="text-sm font-bold uppercase tracking-wider">Lyrics</h2>
      </div>
      <div class="bg-card rounded-2xl border border-foreground/[0.06] divide-y divide-foreground/[0.06]">
        <div class="p-5 flex items-center justify-between">
          <div>
            <p class="text-sm font-semibold">LRCLIB.NET</p>
            <p class="text-xs text-foreground opacity-60 mt-1">Fetch lyrics from LRCLIB.NET</p>
          </div>
          <button @click="appStore.updateEnableLrclib(!appStore.enableLrclib)"
            :class="['w-11 h-6 rounded-full transition-colors relative', appStore.enableLrclib ? 'bg-primary' : 'bg-foreground/20']">
            <span :class="['absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform', appStore.enableLrclib ? 'translate-x-5.5 left-0.5' : 'left-0.5']" />
          </button>
        </div>
        <div class="p-5 flex items-center justify-between">
          <div>
            <p class="text-sm font-semibold">KuGou Lyrics</p>
            <p class="text-xs text-foreground opacity-60 mt-1">Fetch lyrics from KuGou Music</p>
          </div>
          <button @click="appStore.updateEnableKugou(!appStore.enableKugou)"
            :class="['w-11 h-6 rounded-full transition-colors relative', appStore.enableKugou ? 'bg-primary' : 'bg-foreground/20']">
            <span :class="['absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform', appStore.enableKugou ? 'translate-x-5.5 left-0.5' : 'left-0.5']" />
          </button>
        </div>
        <div class="p-5 flex items-center justify-between">
          <div>
            <p class="text-sm font-semibold">Prefer Local Lyrics</p>
            <p class="text-xs text-foreground opacity-60 mt-1">Use local lyric files when available</p>
          </div>
          <button @click="appStore.updatePreferLocalLyrics(!appStore.preferLocalLyrics)"
            :class="['w-11 h-6 rounded-full transition-colors relative', appStore.preferLocalLyrics ? 'bg-primary' : 'bg-foreground/20']">
            <span :class="['absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform', appStore.preferLocalLyrics ? 'translate-x-5.5 left-0.5' : 'left-0.5']" />
          </button>
        </div>
        <div class="p-5 flex items-center justify-between">
          <div>
            <p class="text-sm font-semibold">Auto-Fetch Lyrics</p>
            <p class="text-xs text-foreground opacity-60 mt-1">Automatically fetch missing lyrics after library scan</p>
          </div>
          <button @click="appStore.updateAutoFetchLyrics(!appStore.autoFetchLyrics)"
            :class="['w-11 h-6 rounded-full transition-colors relative', appStore.autoFetchLyrics ? 'bg-primary' : 'bg-foreground/20']">
            <span :class="['absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform', appStore.autoFetchLyrics ? 'translate-x-5.5 left-0.5' : 'left-0.5']" />
          </button>
        </div>
      </div>
    </section>
  </div>
</template>
