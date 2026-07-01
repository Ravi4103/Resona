<script setup lang="ts">
defineProps<{
  open: boolean
  title: string
  message: string
  confirmLabel?: string
  cancelLabel?: string
  danger?: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'confirm': []
  'cancel': []
}>()

function handleCancel() {
  emit('update:open', false)
  emit('cancel')
}

function handleConfirm() {
  emit('update:open', false)
  emit('confirm')
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div v-if="open" class="fixed inset-0 z-50 flex items-center justify-center">
        <div class="absolute inset-0 bg-background/60 backdrop-blur-sm" @click="handleCancel" />
        <div class="relative z-10 rounded-3xl bg-glass-modal backdrop-blur-xl ring-1 ring-border-glass shadow-2xl p-5 w-72"
          @keydown.esc="handleCancel">
          <h3 class="text-base font-semibold text-foreground mb-4">{{ title }}</h3>
          <p class="text-sm text-foreground/70 leading-relaxed mb-4">{{ message }}</p>
          <div class="flex gap-2 justify-end">
            <button @click="handleCancel"
              class="px-3 py-1.5 text-sm font-medium rounded-lg hover:bg-foreground/5 transition-colors">
              {{ cancelLabel ?? 'Cancel' }}
            </button>
            <button @click="handleConfirm"
              :class="[
                'px-3 py-1.5 text-sm font-medium rounded-lg transition-colors',
                danger ? 'bg-error text-white hover:bg-error/80' : 'bg-primary text-white hover:bg-primary/90'
              ]">
              {{ confirmLabel ?? 'Confirm' }}
            </button>
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
