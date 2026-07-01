<script setup lang="ts">
withDefaults(defineProps<{
  variant?: 'default' | 'glass'
}>(), { variant: 'default' })
</script>

<template>
  <div class="rounded-xl overflow-hidden ring-1 ring-foreground/[0.06]"
    :class="variant === 'glass' ? 'dark' : ''">
    <slot />
  </div>
</template>
