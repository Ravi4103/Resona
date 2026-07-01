<script setup lang="ts">
import { Search, Settings } from 'lucide-vue-next'
import SidebarNav from './SidebarNav.vue'
import SidebarPlaylists from './SidebarPlaylists.vue'
import SidebarItem from './SidebarItem.vue'
</script>

<template>
  <div class="flex flex-col h-full bg-background w-full">
    <div class="px-3 py-2">
      <SidebarItem to="/search" :icon="Search" label="Search" />
    </div>

    <div class="mx-3 border-t border-foreground/[0.06] mb-1" />

    <SidebarNav />

    <div class="mx-3 border-t border-foreground/[0.06] my-1" />

    <SidebarPlaylists />

    <div class="mx-3 border-t border-foreground/[0.06] my-1" />

    <div class="px-3 py-2">
      <SidebarItem to="/settings" :icon="Settings" label="Settings" />
    </div>
  </div>
</template>
