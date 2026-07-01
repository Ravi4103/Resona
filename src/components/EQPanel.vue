<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { MoreHorizontal, Plus, Pencil, Trash2 } from 'lucide-vue-next'
import { useAppStore } from '../stores/app'
import * as api from '../lib/invoke'
import type { EQProfile } from '../lib/invoke'
import Slider from './Slider.vue'
import ContextMenu from './ContextMenu.vue'
import { useContextMenu } from '../composables/useContextMenu'
import EQProfileDialog from './EQProfileDialog.vue'
import {
  SelectRoot as Select,
  SelectContent,
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  SelectPortal,
  SelectTrigger,
  SelectValue,
  SelectViewport,
} from 'radix-vue'
import { Check } from 'lucide-vue-next'

const app = useAppStore()

const EQ_FREQ_LABELS = ['32', '64', '125', '250', '500', '1k', '2k', '4k', '8k', '16k']

const profiles = ref<EQProfile[]>([])
const activeProfile = ref<EQProfile | null>(null)
const showAddModal = ref(false)
const showRenameModal = ref(false)
const deleting = ref(false)
const contextMenu = useContextMenu()

onMounted(async () => {
  try {
    const [all, active] = await Promise.all([
      api.getEqProfiles(),
      api.getActiveEqProfile(),
    ])
    profiles.value = all
    if (active) {
      activeProfile.value = all.find((p) => p.id === active.id) || active
    }
  } catch (e) {
    console.error('Failed to load EQ profiles', e)
  }
})

const bands = computed(() => {
  if (!activeProfile.value) return []
  return activeProfile.value.bands.map((gain, i) => ({ freq: EQ_FREQ_LABELS[i], gain }))
})

async function selectProfile(id: string) {
  await api.applyEqProfile(id)
  await app.updateEqEnabled(true)
  profiles.value = profiles.value.map((p) => ({ ...p, is_active: p.id === id }))
  const p = profiles.value.find((x) => x.id === id)
  if (p) activeProfile.value = p
}

function onBandInput(bandIndex: number, value: number) {
  if (!activeProfile.value) return
  const bands = [...activeProfile.value.bands]
  bands[bandIndex] = value
  activeProfile.value = { ...activeProfile.value, bands }
}

async function onBandCommit(bandIndex: number) {
  if (!activeProfile.value) return
  const gain = activeProfile.value.bands[bandIndex]
  await api.updateEqBand(activeProfile.value.id, bandIndex, gain)
  await api.setEqBand(bandIndex, gain)
}

async function toggleEnabled() {
  const enabled = !app.eqEnabled
  app.updateEqEnabled(enabled)
  await api.setEqEnabled(enabled)
}

function getBandGain(index: number): number {
  return activeProfile.value?.bands[index] ?? 0
}

function openAddModal() {
  showAddModal.value = true
}

async function createProfile(name: string) {
  const p = await api.createEqProfile(name)
  if (p) {
    profiles.value.push(p)
    await selectProfile(p.id)
  }
}

function openRenameModal() {
  if (!activeProfile.value) return
  showRenameModal.value = true
}

async function renameProfile(name: string) {
  if (!activeProfile.value) return
  await api.renameEqProfile(activeProfile.value.id, name)
  profiles.value = profiles.value.map((p) =>
    p.id === activeProfile.value!.id ? { ...p, name } : p
  )
  const updated = profiles.value.find((p) => p.id === activeProfile.value!.id)
  if (updated) activeProfile.value = updated
}

async function deleteProfile() {
  if (!activeProfile.value || activeProfile.value.is_default) return
  deleting.value = true
  try {
    const deletedId = activeProfile.value.id
    await api.deleteEqProfile(deletedId)
    profiles.value = profiles.value.filter((p) => p.id !== deletedId)
    const fallback = profiles.value[0]
    if (fallback) await selectProfile(fallback.id)
    else activeProfile.value = null
  } finally {
    deleting.value = false
  }
}

function openProfileMenu(e: MouseEvent) {
  const isUserProfile = activeProfile.value && !activeProfile.value.is_default
  contextMenu.open(e, [
    {
      label: 'New Profile',
      icon: Plus,
      action: openAddModal,
    },
    ...(isUserProfile
      ? [
          { separator: true as const },
          {
            label: 'Rename Profile',
            icon: Pencil,
            action: openRenameModal,
          },
          {
            label: 'Delete Profile',
            icon: Trash2,
            danger: true,
            action: deleteProfile,
          },
        ]
      : []),
  ])
}

function onBandMouseUp(i: number) {
  onBandCommit(i)
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center gap-2">
      <button
        class="flex items-center gap-2 px-3 py-1.5 rounded-3xl text-sm transition-colors w-18 h-10 border border-foreground/[0.08]"
        :class="app.eqEnabled
          ? 'bg-foreground/[0.1] text-foreground hover:bg-foreground/[0.14]'
          : 'bg-foreground/[0.03] text-foreground opacity-60 hover:bg-foreground/[0.06]'"
        @click="toggleEnabled">
        <span class="w-1.5 h-1.5 rounded-full" :class="app.eqEnabled ? 'bg-green-400' : 'bg-foreground/20'" />
        {{ app.eqEnabled ? 'On' : 'Off' }}
      </button>
      <Select v-if="profiles.length > 0" :model-value="activeProfile?.id" @update:model-value="selectProfile">
        <SelectTrigger
          class="flex-1 bg-foreground/[0.05] border border-foreground/[0.08] text-sm text-foreground rounded-2xl px-3 py-1.5 focus:outline-none focus:ring-1 focus:ring-foreground/20">
          <SelectValue :placeholder="'Select Profile'" />
        </SelectTrigger>
        <SelectPortal>
          <SelectContent
            class="relative z-50 min-w-[8rem] overflow-hidden rounded-2xl border border-border-glass bg-glass-elevated backdrop-blur-xl text-foreground shadow-2xl transform-gpu isolate data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2">
            <SelectViewport class="p-1.5 w-full min-w-[var(--radix-select-trigger-width)]">
              <div class="overflow-x-hidden overflow-y-auto max-h-[min(var(--radix-select-content-available-height),20rem)]">
                <SelectItem v-for="p in profiles" :key="p.id" :value="p.id"
                  class="relative flex w-full cursor-default select-none items-center rounded-lg py-2 pl-8 pr-2 text-xs outline-none focus:bg-foreground/15 data-[disabled]:pointer-events-none data-[disabled]:opacity-50 mx-0.5 my-0.5 data-[state=checked]:text-primary"
                  :class="p.is_active ? 'text-primary' : 'text-foreground opacity-80'">
                  <span class="absolute left-2 flex h-3.5 w-3.5 items-center justify-center">
                    <SelectItemIndicator>
                      <Check class="h-4 w-4" />
                    </SelectItemIndicator>
                  </span>
                  <SelectItemText>{{ p.name }}</SelectItemText>
                </SelectItem>
              </div>
            </SelectViewport>
          </SelectContent>
        </SelectPortal>
      </Select>
      <button @click="openProfileMenu"
        class="flex items-center justify-center w-10 h-10 rounded-full text-sm transition-colors border border-foreground/[0.08] bg-foreground/[0.05] text-foreground opacity-80 hover:bg-foreground/[0.1] hover:text-foreground">
        <MoreHorizontal class="w-4 h-4" />
      </button>
    </div>

    <div v-if="app.eqEnabled" class="flex items-end justify-between gap-1 h-40 px-1">
      <div v-for="(band, i) in bands" :key="i" class="flex flex-col items-center flex-1 min-w-0 h-full">
        <p class="text-[10px] text-foreground opacity-80 mb-1 tabular-nums w-full text-center">
          {{ band.gain >= 0 ? '+' : '' }}{{ band.gain.toFixed(1) }}
        </p>
        <div class="flex-1 flex items-center justify-center w-full">
          <div class="relative" style="width: 24px; height: 80px;">
            <div class="absolute inset-0 flex items-center justify-center"
              style="transform: rotate(-90deg); transform-origin: center; width: 80px; height: 24px; top: 50%; left: 50%; margin-top: -12px; margin-left: -40px;">
              <Slider :model-value="getBandGain(i)" :min="-12" :max="12" :step="0.5" class="w-full"
                @update:model-value="(val: number) => onBandInput(i, val)"
                @mouseup="() => onBandMouseUp(i)"
                @touchend="() => onBandMouseUp(i)" />
            </div>
          </div>
        </div>
        <p class="text-[10px] text-foreground opacity-80 mt-1">{{ band.freq }}</p>
      </div>
    </div>

  </div>

  <ContextMenu
    :visible="contextMenu.visible.value"
    :x="contextMenu.x.value"
    :y="contextMenu.y.value"
    :items="contextMenu.items.value"
    @close="contextMenu.close()" />

  <EQProfileDialog
    :open="showAddModal"
    title="New Profile"
    confirm-label="Add Profile"
    @update:open="showAddModal = $event"
    @confirm="createProfile" />

  <EQProfileDialog
    :open="showRenameModal"
    title="Rename Profile"
    confirm-label="Save"
    :initial-name="activeProfile?.name"
    @update:open="showRenameModal = $event"
    @confirm="renameProfile" />
</template>
