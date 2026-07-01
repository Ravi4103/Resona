<script setup lang="ts">
import { AudioLines, Settings2 } from 'lucide-vue-next'
import EQPanel from '../EQPanel.vue'
import { useAppStore } from '../../stores/app'

const appStore = useAppStore()
</script>

<template>
  <div class="space-y-10 animate-in fade-in slide-in-from-bottom-2 duration-500">
    <section>
      <div class="flex items-center gap-2 mb-6 text-foreground opacity-60 select-none">
        <Settings2 class="w-4 h-4" />
        <h2 class="text-sm font-bold uppercase tracking-wider">General</h2>
      </div>
      <div class="bg-card rounded-2xl border border-foreground/[0.06] divide-y divide-foreground/[0.06]">
        <div class="p-5 flex items-center justify-between gap-x-2">
          <div>
            <p class="text-sm font-semibold">Prevent Sleep</p>
            <p class="text-xs text-foreground opacity-60 mt-1">Prevent system from sleeping while playing</p>
          </div>
          <button @click="appStore.updatePreventSleepWhilePlaying(!appStore.preventSleepWhilePlaying)"
            :class="['w-11 h-6 rounded-full transition-colors relative', appStore.preventSleepWhilePlaying ? 'bg-primary' : 'bg-foreground/20']">
            <span :class="['absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform', appStore.preventSleepWhilePlaying ? 'translate-x-5.5 left-0.5' : 'left-0.5']" />
          </button>
        </div>
        <div class="p-5 flex items-center justify-between gap-x-2">
          <div>
            <p class="text-sm font-semibold">Player Indicator</p>
            <p class="text-xs text-foreground opacity-60 mt-1">Show player indicator in sidebar</p>
          </div>
          <button @click="appStore.updateShowPlayerIndicator(!appStore.showPlayerIndicator)"
            :class="['w-11 h-6 rounded-full transition-colors relative', appStore.showPlayerIndicator ? 'bg-primary' : 'bg-foreground/20']">
            <span :class="['absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform', appStore.showPlayerIndicator ? 'translate-x-5.5 left-0.5' : 'left-0.5']" />
          </button>
        </div>
      </div>
    </section>

    <section>
      <div class="flex items-center gap-2 mb-6 text-foreground opacity-60 select-none">
        <AudioLines class="w-4 h-4" />
        <h2 class="text-sm font-bold uppercase tracking-wider">Equalizer</h2>
      </div>
      <div class="bg-card rounded-2xl border border-foreground/[0.06] p-6">
        <EQPanel />
      </div>
    </section>
  </div>
</template>
