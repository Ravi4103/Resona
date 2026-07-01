<script setup lang="ts">
import { computed } from 'vue'
import { useAppStore } from '../../stores/app'
import { Copy, Dices, Wifi, Info } from 'lucide-vue-next'

const appStore = useAppStore()

const remoteUrl = computed(() => {
  const host = window.location.hostname || 'localhost'
  return `http://${host}:${appStore.remoteServerPort}`
})

function copyRemoteUrl() {
  navigator.clipboard.writeText(remoteUrl.value).catch(() => {})
}

function regeneratePin() {
  appStore.updateRemoteServerPassword(String(Math.floor(1000 + Math.random() * 9000)))
}
</script>

<template>
  <div class="space-y-10 animate-in fade-in slide-in-from-bottom-2 duration-500">
    <section>
      <div class="flex items-center gap-2 mb-6 text-foreground opacity-60 select-none">
        <Wifi class="w-4 h-4" />
        <h2 class="text-sm font-bold uppercase tracking-wider">Remote Control</h2>
      </div>
      <div class="bg-card rounded-2xl border border-foreground/[0.06] divide-y divide-foreground/[0.06]">
        <div class="p-5 flex items-center justify-between gap-x-2">
          <div>
            <p class="text-sm font-semibold">Enable Remote Server</p>
            <p class="text-xs text-foreground opacity-60 mt-1">Control this player from other devices on your network</p>
          </div>
          <button @click="appStore.updateRemoteServerEnabled(!appStore.remoteServerEnabled)"
            :class="['w-11 h-6 rounded-full transition-colors relative', appStore.remoteServerEnabled ? 'bg-primary' : 'bg-foreground/20']">
            <span :class="['absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform', appStore.remoteServerEnabled ? 'translate-x-5.5 left-0.5' : 'left-0.5']" />
          </button>
        </div>

        <div class="px-5 py-3 flex items-start gap-2 text-xs text-foreground opacity-60">
          <Info class="w-3 h-3 mt-0.5 shrink-0" />
          <span>Configure your firewall to allow incoming connections on port {{ appStore.remoteServerPort }}</span>
        </div>

        <template v-if="appStore.remoteServerEnabled">
          <div class="p-5">
            <p class="text-sm font-semibold mb-3">Server URL</p>
            <div class="flex items-center gap-2">
              <code class="flex-1 px-3 py-2 text-xs rounded-xl bg-foreground/[0.02] border border-foreground/[0.04] truncate">{{ remoteUrl }}</code>
              <button @click="copyRemoteUrl" class="p-2 text-foreground/50 hover:text-foreground rounded-lg" title="Copy URL">
                <Copy class="w-4 h-4" />
              </button>
            </div>
          </div>

          <div class="p-5 flex items-start justify-between gap-x-2">
            <div>
              <p class="text-sm font-semibold">Access PIN</p>
              <p class="text-xs text-foreground opacity-60 mt-1">4-digit PIN required to connect</p>
            </div>
            <div class="flex items-center gap-4">
              <input v-model="appStore.remoteServerPassword" type="text" maxlength="4"
                class="w-20 px-3 py-2 text-lg font-mono text-center rounded-xl bg-foreground/[0.04] border border-foreground/10 outline-none focus:border-primary/50" />
              <button @click="regeneratePin"
                class="p-2 text-foreground/50 hover:text-foreground rounded-lg">
                <Dices class="w-4 h-4" />
              </button>
            </div>
          </div>
        </template>
      </div>
    </section>
  </div>
</template>
