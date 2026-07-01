<script setup lang="ts">
import { useLibraryStore } from '../../stores/library'

const APP_VERSION = '0.1.0'
const library = useLibraryStore()
</script>

<template>
  <div class="space-y-8 animate-in fade-in slide-in-from-bottom-2 duration-500">
    <div class="text-center py-8">
      <div class="w-24 h-24 mx-auto mb-6 rounded-2xl bg-primary/10 flex items-center justify-center shadow-2xl">
        <span class="text-4xl font-black text-primary">♪</span>
      </div>
      <h2 class="text-3xl font-black mb-2">Resoná</h2>
      <p class="text-sm font-bold text-primary mb-2">Version {{ APP_VERSION }}</p>
    </div>

    <section>
      <div class="bg-card rounded-2xl border border-foreground/[0.06] p-6 space-y-4">
        <p class="text-sm text-foreground/70 leading-relaxed">
          A cross-platform offline music player with local library management, gapless playback, lyrics, playlists, internet radio, and remote control.
        </p>
        <div class="pt-4 border-t border-foreground/[0.06] flex items-center justify-between text-sm">
          <span class="text-foreground/50">{{ library.tracks.length }} tracks</span>
          <span class="text-foreground/50">{{ library.playlists.length }} playlists</span>
        </div>
      </div>
    </section>
  </div>
</template>
