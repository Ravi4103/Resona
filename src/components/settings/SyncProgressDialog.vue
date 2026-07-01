<script setup lang="ts">
import { Loader2, CheckCircle2 } from 'lucide-vue-next'
import ConfirmDialog from '../ConfirmDialog.vue'

defineProps<{
  open: boolean
  scanning: boolean
  scanResult: string | null
}>()

const emit = defineEmits<{
  cancel: []
}>()
</script>

<template>
  <ConfirmDialog
    :open="open"
    title="Library Sync"
    :message="scanResult || 'Syncing library...'"
    confirm-label="OK"
    :danger="false"
    @confirm="emit('cancel')"
    @cancel="emit('cancel')"
  >
    <template #icon>
      <CheckCircle2 v-if="scanResult && !scanResult.startsWith('Error')" class="w-10 h-10 text-primary" />
      <Loader2 v-else class="w-10 h-10 animate-spin text-primary" />
    </template>
  </ConfirmDialog>
</template>
