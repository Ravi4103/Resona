<script setup lang="ts">
import { ref, watch } from 'vue'

const props = withDefaults(defineProps<{
  open: boolean
  trackCount: number
}>(), { trackCount: 1 })

const emit = defineEmits<{
  confirm: [position: 'next' | 'end']
  close: []
}>()

const position = ref<'next' | 'end'>('end')

watch(() => props.open, (val) => {
  if (val) position.value = 'end'
})
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="fixed inset-0 z-[999] flex items-center justify-center" @click.self="emit('close')">
      <div class="absolute inset-0 bg-background/60 backdrop-blur-sm" />
      <div class="relative bg-background border border-foreground/[0.08] rounded-2xl shadow-2xl w-full max-w-sm p-6 space-y-5">
        <div>
          <h3 class="text-lg font-bold">Add to Queue</h3>
          <p class="text-sm text-foreground opacity-60 mt-1">{{ trackCount }} track{{ trackCount === 1 ? '' : 's' }}</p>
        </div>

        <div class="space-y-2">
          <label class="flex items-center gap-3 p-3 rounded-xl cursor-pointer transition-colors"
            :class="position === 'end' ? 'bg-primary/10 ring-1 ring-primary/30' : 'hover:bg-foreground/[0.04]'">
            <input type="radio" value="end" v-model="position" class="sr-only" />
            <div class="w-4 h-4 rounded-full border-2 flex items-center justify-center"
              :class="position === 'end' ? 'border-primary' : 'border-foreground/30'">
              <div v-if="position === 'end'" class="w-2 h-2 rounded-full bg-primary" />
            </div>
            <div>
              <p class="text-sm font-medium">Add to End</p>
              <p class="text-xs text-foreground opacity-50">Append to the end of the queue</p>
            </div>
          </label>
          <label class="flex items-center gap-3 p-3 rounded-xl cursor-pointer transition-colors"
            :class="position === 'next' ? 'bg-primary/10 ring-1 ring-primary/30' : 'hover:bg-foreground/[0.04]'">
            <input type="radio" value="next" v-model="position" class="sr-only" />
            <div class="w-4 h-4 rounded-full border-2 flex items-center justify-center"
              :class="position === 'next' ? 'border-primary' : 'border-foreground/30'">
              <div v-if="position === 'next'" class="w-2 h-2 rounded-full bg-primary" />
            </div>
            <div>
              <p class="text-sm font-medium">Play Next</p>
              <p class="text-xs text-foreground opacity-50">Insert after the currently playing track</p>
            </div>
          </label>
        </div>

        <div class="flex gap-3 pt-1">
          <button @click="emit('close')"
            class="flex-1 px-4 py-2.5 rounded-xl bg-foreground/[0.06] text-foreground font-medium hover:bg-foreground/[0.1] transition-colors text-sm">
            Cancel
          </button>
          <button @click="emit('confirm', position)"
            class="flex-1 px-4 py-2.5 rounded-xl bg-primary text-primary-foreground font-medium hover:opacity-90 transition-opacity text-sm shadow-lg shadow-primary/20">
            Add
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
