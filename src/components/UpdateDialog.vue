<script setup lang="ts">

import { Sparkles } from 'lucide-vue-next'

const props = defineProps<{
  open: boolean
  version?: string
  releaseNotes?: string
  isUpdating?: boolean
  updateApplied?: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'update': []
  'restart': []
}>()

async function handleUpdate() {
  emit('update')
}

function handleRestart() {
  emit('restart')
}

function close() {
  if (props.isUpdating) return
  emit('update:open', false)
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div v-if="open" class="fixed inset-0 z-50 flex items-center justify-center">
        <div class="absolute inset-0 bg-background/60 backdrop-blur-sm" @click="close" />
        <div class="relative z-10 rounded-3xl bg-glass-modal backdrop-blur-xl ring-1 ring-border-glass shadow-2xl p-5 w-[500px] max-h-[80vh] flex flex-col"
          @keydown.esc="close">
          <div class="flex items-center gap-2 mb-4">
            <Sparkles class="w-5 h-5 text-primary" />
            <h3 class="text-base font-semibold text-foreground">
              Update Available {{ version ? `(v${version})` : '' }}
            </h3>
          </div>

          <div class="flex-1 overflow-y-auto min-h-0 my-4 text-sm text-foreground/80 leading-relaxed custom-scrollbar">
            <template v-if="updateApplied">
              <div class="h-full flex flex-col items-center justify-center text-center py-8">
                <div class="w-16 h-16 bg-green-500/10 text-green-500 rounded-full flex items-center justify-center mb-4">
                  <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M20 6L9 17l-5-5"/>
                  </svg>
                </div>
                <p class="text-base font-medium text-foreground mb-2">Update Applied Successfully</p>
                <p class="text-sm text-foreground/60">Please restart the application to complete the update.</p>
              </div>
            </template>
            <div v-else class="prose prose-invert prose-sm max-w-none">
              <div v-if="releaseNotes" v-html="releaseNotes" />
              <p v-else class="text-foreground/60">No release notes available.</p>
            </div>
          </div>

          <div class="flex justify-end gap-3 pt-2">
            <button
              v-if="!updateApplied"
              @click="close"
              class="px-4 py-2 text-sm font-medium rounded-lg hover:bg-foreground/5 transition-colors disabled:opacity-50"
              :disabled="isUpdating"
            >
              Later
            </button>
            <button
              v-if="!updateApplied"
              @click="handleUpdate()"
              class="px-4 py-2 text-sm font-medium rounded-lg bg-primary text-white hover:bg-primary/90 transition-colors disabled:opacity-50 flex items-center gap-2"
              :disabled="isUpdating"
            >
              <div v-if="isUpdating" class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
              {{ isUpdating ? 'Updating...' : 'Update Now' }}
            </button>
            <template v-if="updateApplied">
              <button
                @click="close"
                class="px-4 py-2 text-sm font-medium rounded-lg hover:bg-foreground/5 transition-colors"
              >
                Later
              </button>
              <button
                @click="handleRestart()"
                class="px-4 py-2 text-sm font-medium rounded-lg bg-primary text-white hover:bg-primary/90 transition-colors flex items-center gap-2"
              >
                Restart Now
              </button>
            </template>
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

.prose :deep(h1),
.prose :deep(h2),
.prose :deep(h3),
.prose :deep(h4) {
  color: var(--foreground, currentColor);
  font-weight: 600;
  line-height: 1.3;
  margin: 1em 0 0.5em;
}
.prose :deep(h1):first-child,
.prose :deep(h2):first-child,
.prose :deep(h3):first-child,
.prose :deep(h4):first-child {
  margin-top: 0;
}
.prose :deep(h1) { font-size: 1.25rem; }
.prose :deep(h2) { font-size: 1.125rem; }
.prose :deep(h3) { font-size: 1rem; }
.prose :deep(h4) { font-size: 0.9375rem; }

.prose :deep(p) { margin: 0.5em 0; }
.prose :deep(p):first-child { margin-top: 0; }

.prose :deep(ul),
.prose :deep(ol) {
  margin: 0.5em 0;
  padding-left: 1.5em;
}
.prose :deep(ul) { list-style: disc; }
.prose :deep(ol) { list-style: decimal; }
.prose :deep(li) { margin: 0.25em 0; }

.prose :deep(a) {
  color: var(--primary, #3b82f6);
  text-decoration: underline;
}

.prose :deep(strong) {
  color: var(--foreground, currentColor);
  font-weight: 600;
}

.prose :deep(code) {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 0.85em;
  background: rgba(127, 127, 127, 0.15);
  padding: 0.15em 0.35em;
  border-radius: 0.25rem;
}
.prose :deep(pre) {
  background: rgba(127, 127, 127, 0.12);
  padding: 0.75em 1em;
  border-radius: 0.5rem;
  overflow-x: auto;
  margin: 0.75em 0;
}
.prose :deep(pre code) {
  background: transparent;
  padding: 0;
}

.prose :deep(blockquote) {
  border-left: 3px solid rgba(127, 127, 127, 0.4);
  padding-left: 0.75em;
  margin: 0.5em 0;
  opacity: 0.85;
}

.prose :deep(hr) {
  border: 0;
  border-top: 1px solid rgba(127, 127, 127, 0.25);
  margin: 1em 0;
}
</style>
