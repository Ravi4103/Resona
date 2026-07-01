<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  DialogRoot,
  DialogPortal,
  DialogOverlay,
  DialogContent,
  DialogTitle,
} from 'radix-vue'

const props = defineProps<{
  open: boolean
  initialName?: string
  title?: string
  confirmLabel?: string
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
  <DialogRoot :open="open" @update:open="(v: boolean) => emit('update:open', v)">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 bg-black/50 backdrop-blur-sm z-[999]" />
      <DialogContent
        class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-[1000] w-[400px] rounded-2xl bg-glass-modal backdrop-blur-2xl ring-1 ring-border-glass shadow-2xl p-6 select-none">
        <DialogTitle class="text-lg font-semibold mb-4">{{ title ?? 'New Playlist' }}</DialogTitle>
        <input v-model="name" placeholder="Playlist name"
          class="w-full px-3 py-2 text-sm rounded-lg bg-foreground/[0.07] border border-foreground/20 text-foreground placeholder:text-foreground/40 outline-none focus:border-primary/50 transition-colors"
          autofocus @keydown.enter="submit" />
        <div class="flex justify-end gap-2 mt-4">
          <button
            class="px-3 py-1.5 text-sm text-foreground opacity-70 hover:text-foreground rounded-lg hover:bg-foreground/[0.05] transition-colors"
            @click="emit('update:open', false)">Cancel</button>
          <button
            class="px-3 py-1.5 text-sm bg-primary text-white rounded-lg transition-colors font-medium disabled:opacity-40"
            :disabled="!name.trim()"
            @click="submit">{{ confirmLabel ?? 'Create' }}</button>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
