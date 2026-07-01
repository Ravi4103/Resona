<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { Settings, Folder, Play, Info, Blocks, Wifi } from 'lucide-vue-next'
import { useAppStore } from '../stores/app'
import GeneralSettings from '../components/settings/GeneralSettings.vue'
import LibrarySettings from '../components/settings/LibrarySettings.vue'
import IntegrationsSettings from '../components/settings/IntegrationsSettings.vue'
import PlaybackSettings from '../components/settings/PlaybackSettings.vue'
import AboutSettings from '../components/settings/AboutSettings.vue'
import RemoteServerSettings from '../components/settings/RemoteServerSettings.vue'

const props = defineProps<{ category?: string }>()
const router = useRouter()
const app = useAppStore()

const activeCategory = ref(props.category || 'general')

watch(() => props.category, (cat) => {
  activeCategory.value = cat || 'general'
})

const categories = computed(() => [
  { id: 'general', name: 'General', icon: Settings },
  { id: 'library', name: 'Library', icon: Folder },
  { id: 'playback', name: 'Playback', icon: Play },
  { id: 'integrations', name: 'Integrations', icon: Blocks },
  { id: 'remote', name: 'Remote', icon: Wifi },
  { id: 'about', name: 'About', icon: Info },
])

function setCategory(id: string) {
  activeCategory.value = id
  router.replace(`/settings/${id}`)
  app.setActiveSettingsTab(id as any)
}
</script>

<template>
  <div class="h-full flex flex-col md:flex-row bg-background text-foreground overflow-hidden">
    <aside class="w-full md:w-56 border-r border-foreground/[0.06] bg-foreground/[0.02] flex-shrink-0 select-none">
      <div class="p-6">
        <h1 class="text-2xl font-bold mb-6 px-2">Settings</h1>
        <nav class="space-y-1">
          <button v-for="cat in categories" :key="cat.id" @click="setCategory(cat.id)"
            :class="['w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all group',
              activeCategory === cat.id
                ? 'bg-primary text-primary-foreground shadow-sm shadow-primary/20'
                : 'text-foreground opacity-80 hover:text-foreground hover:bg-foreground/[0.04]'
            ]">
            <component :is="cat.icon" class="w-4 h-4" />
            {{ cat.name }}
          </button>
        </nav>
      </div>
    </aside>

    <main class="flex-1 overflow-y-auto custom-scrollbar">
      <div class="max-w-3xl p-8 mx-auto">
        <GeneralSettings v-if="activeCategory === 'general'" />
        <LibrarySettings v-if="activeCategory === 'library'" />
        <PlaybackSettings v-if="activeCategory === 'playback'" />
        <IntegrationsSettings v-if="activeCategory === 'integrations'" />
        <RemoteServerSettings v-if="activeCategory === 'remote'" />
        <AboutSettings v-if="activeCategory === 'about'" />
      </div>
    </main>
  </div>
</template>
