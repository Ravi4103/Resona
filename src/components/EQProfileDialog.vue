<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps<{
  open: boolean
  title: string
  confirmLabel: string
  initialName?: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  confirm: [name: string]
}>()

const name = ref(props.initialName ?? '')

watch(() => props.open, (val) => {
  if (val) name.value = props.initialName ?? ''
})

function submit() {
  if (!name.value.trim()) return
  emit('confirm', name.value.trim())
  emit('update:open', false)
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div v-if="open" class="fixed inset-0 z-50 flex items-center justify-center">
        <div class="absolute inset-0 bg-background/60 backdrop-blur-sm" @click="emit('update:open', false)" />
        <div class="relative z-10 rounded-3xl bg-glass-modal backdrop-blur-xl ring-1 ring-border-glass shadow-2xl p-5 w-72"
          @keydown.esc="emit('update:open', false)">
          <h3 class="text-base font-semibold text-foreground mb-4">{{ title }}</h3>
          
          <input
            v-model="name"
            type="text"
            placeholder="Profile name"
            class="w-full px-3 py-2 rounded-lg bg-foreground/[0.07] border border-foreground/20 text-foreground placeholder:text-foreground/40 focus-visible:ring-2 focus-visible:ring-primary/20 outline-none transition-all"
            autofocus
            @keydown.enter="submit"
          />
          
          <div class="flex justify-end gap-2 mt-4">
            <button
              class="px-3 py-1.5 text-sm text-foreground opacity-70 hover:text-foreground rounded-lg hover:bg-foreground/[0.05] transition-colors"
              @click="emit('update:open', false)">Cancel</button>
            <button
              class="px-3 py-1.5 text-sm text-foreground bg-foreground/[0.12] hover:bg-foreground/[0.18] rounded-lg transition-colors font-medium disabled:opacity-40"
              :disabled="!name.trim()"
              @click="submit">{{ confirmLabel }}</button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-fade-enter-active { transition: opacity 0.2s ease-out; }
.modal-fade-leave-active { transition: opacity 0.2s ease-in; }
.modal-fade-enter-from, .modal-fade-leave-to { opacity: 0; }
.modal-fade-enter-active .relative {
  transition: transform 0.3s ease-out, opacity 0.3s ease-out;
  transition-delay: 0.05s;
}
.modal-fade-enter-from .relative {
  transform: scale(0.95);
  opacity: 0;
}
.modal-fade-leave-active .relative {
  transition: transform 0.2s ease-in, opacity 0.2s ease-in;
}
.modal-fade-leave-to .relative {
  transform: scale(0.98);
  opacity: 0;
}
</style>
