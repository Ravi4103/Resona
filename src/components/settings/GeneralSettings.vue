<script setup lang="ts">
import { AppWindow, Sun, Moon, Monitor, Circle, Palette } from 'lucide-vue-next'
import { useAppStore } from '../../stores/app'
import ColorPicker from '../ColorPicker.vue'

const appStore = useAppStore()

const themes = [
  { value: 'system' as const, icon: Monitor, label: 'System' },
  { value: 'light' as const, icon: Sun, label: 'Light' },
  { value: 'dark' as const, icon: Moon, label: 'Dark' },
  { value: 'black' as const, icon: Circle, label: 'Black' },
]
</script>

<template>
  <div class="space-y-10 animate-in fade-in slide-in-from-bottom-2 duration-500">
    <section>
      <div class="flex items-center gap-2 mb-6 text-foreground opacity-60 select-none">
        <AppWindow class="w-4 h-4" />
        <h2 class="text-sm font-bold uppercase tracking-wider">Behavior</h2>
      </div>
      <div class="bg-card rounded-2xl border border-foreground/[0.06] divide-y divide-foreground/[0.06]">
        <div class="p-5 flex items-center justify-between gap-x-2">
          <div>
            <p class="text-sm font-semibold">Start at Login</p>
            <p class="text-xs text-foreground opacity-60 mt-1">Launch automatically when you sign in</p>
          </div>
          <button @click="appStore.updateStartAtLogin(!appStore.startAtLogin)"
            :class="['w-11 h-6 rounded-full transition-colors relative', appStore.startAtLogin ? 'bg-primary' : 'bg-foreground/20']">
            <span :class="['absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform', appStore.startAtLogin ? 'translate-x-5.5 left-0.5' : 'left-0.5']" />
          </button>
        </div>
        <div class="p-5 flex items-center justify-between gap-x-2">
          <div>
            <p class="text-sm font-semibold">Show Tray Icon</p>
            <p class="text-xs text-foreground opacity-60 mt-1">Display icon in system tray (requires restart)</p>
          </div>
          <button @click="appStore.updateShowTrayIcon(!appStore.showTrayIcon)"
            :class="['w-11 h-6 rounded-full transition-colors relative', appStore.showTrayIcon ? 'bg-primary' : 'bg-foreground/20']">
            <span :class="['absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform', appStore.showTrayIcon ? 'translate-x-5.5 left-0.5' : 'left-0.5']" />
          </button>
        </div>
      </div>
    </section>

    <section>
      <div class="flex items-center gap-2 mb-6 text-foreground opacity-60 select-none">
        <Sun class="w-4 h-4" />
        <h2 class="text-sm font-bold uppercase tracking-wider">Appearance</h2>
      </div>
      <div class="bg-card rounded-2xl border border-foreground/[0.06]">
        <div class="p-5 flex items-center justify-between">
          <div class="flex items-center gap-4">
            <div class="p-2 bg-foreground/[0.04] rounded-xl">
              <Sun v-if="appStore.theme === 'light'" class="w-5 h-5 text-foreground opacity-80" />
              <Moon v-else-if="appStore.theme === 'dark'" class="w-5 h-5 text-foreground opacity-80" />
              <Circle v-else-if="appStore.theme === 'black'" class="w-5 h-5 text-foreground opacity-80" />
              <Monitor v-else class="w-5 h-5 text-foreground opacity-80" />
            </div>
            <div>
              <p class="text-sm font-semibold">Theme</p>
              <p class="text-xs text-foreground opacity-60 mt-1">Choose your preferred theme</p>
            </div>
          </div>
          <div class="flex gap-1.5">
            <button v-for="t in themes" :key="t.value" @click="appStore.updateTheme(t.value)"
              :class="['p-2.5 rounded-xl border transition-all text-xs', appStore.theme === t.value ? 'bg-primary text-primary-foreground border-primary shadow-sm shadow-primary/20' : 'bg-foreground/[0.04] border-foreground/10 text-foreground/70 hover:text-foreground']">
              <component :is="t.icon" class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </section>

    <section>
      <div class="flex items-center gap-2 mb-6 text-foreground opacity-60 select-none">
        <Palette class="w-4 h-4" />
        <h2 class="text-sm font-bold uppercase tracking-wider">Colors</h2>
      </div>
      <div class="bg-card rounded-2xl border border-foreground/[0.06] divide-y divide-foreground/[0.06]">
        <div class="p-5 space-y-4">
          <ColorPicker label="Primary Accent" color-key="--primary" />
          <ColorPicker label="Favorite" color-key="--favorite" />
          <ColorPicker label="Success" color-key="--success" />
          <ColorPicker label="Warning" color-key="--warning" />
          <ColorPicker label="Error" color-key="--error" />
          <ColorPicker label="Info" color-key="--info" />
          <ColorPicker label="Secondary Accent" color-key="--accent-secondary" />
          <ColorPicker label="Now Playing" color-key="--now-playing" />
        </div>
        <div class="p-5">
          <button @click="appStore.resetColors()"
            class="text-sm text-foreground/60 hover:text-foreground transition-colors underline underline-offset-2">
            Reset all colors to defaults
          </button>
        </div>
      </div>
    </section>
  </div>
</template>
