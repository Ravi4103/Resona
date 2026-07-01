<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'

const props = defineProps<{
  text?: string
  containerClass?: string
  contentClass?: string
  pauseOnHover?: boolean
}>()

const containerRef = ref<HTMLElement | null>(null)
const contentRef = ref<HTMLElement | null>(null)
const isOverflowing = ref(false)
const scrollDistance = ref(0)
const duration = ref(0)

const updateMarquee = async () => {
  await nextTick()
  if (!containerRef.value || !contentRef.value) return

  isOverflowing.value = false
  await nextTick()

  const containerWidth = containerRef.value.offsetWidth
  const contentWidth = contentRef.value.scrollWidth

  if (contentWidth > containerWidth + 1) {
    isOverflowing.value = true
    scrollDistance.value = contentWidth - containerWidth
    duration.value = (scrollDistance.value / 20) + 4
  } else {
    isOverflowing.value = false
    scrollDistance.value = 0
  }
}

let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  updateMarquee()
  resizeObserver = new ResizeObserver(() => { updateMarquee() })
  if (containerRef.value) resizeObserver.observe(containerRef.value)
})

onUnmounted(() => {
  if (resizeObserver) resizeObserver.disconnect()
})

watch(() => props.text, () => { updateMarquee() })
</script>

<template>
  <div
    ref="containerRef"
    :class="['overflow-hidden whitespace-nowrap min-w-0 w-full', containerClass].filter(Boolean).join(' ')"
    :title="text"
  >
    <div
      ref="contentRef"
      :class="[
        'w-fit min-w-full inline-block',
        isOverflowing && 'marquee-anim',
        pauseOnHover && 'hover:[animation-play-state:paused]',
        contentClass
      ].filter(Boolean).join(' ')"
      :style="{
        '--scroll-dist': `-${scrollDistance}px`,
        '--duration': `${duration}s`
      }"
    >
      <slot>{{ text }}</slot>
    </div>
  </div>
</template>

<style scoped>
.marquee-anim {
  animation: pingpong var(--duration) ease-in-out infinite;
}

@keyframes pingpong {
  0%, 15% { transform: translateX(0); }
  45%, 55% { transform: translateX(var(--scroll-dist)); }
  85%, 100% { transform: translateX(0); }
}
</style>
