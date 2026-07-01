<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { RecycleScroller } from 'vue-virtual-scroller'

const props = withDefaults(defineProps<{
  items: any[]
  minColumnWidth?: number
  gap?: number
  squareItems?: boolean
  textAreaHeight?: number
  cardHeight?: number
}>(), {
  minColumnWidth: 180,
  gap: 24,
  squareItems: true,
  textAreaHeight: 60,
  cardHeight: 0,
})

const containerRef = ref<HTMLElement | null>(null)
const containerWidth = ref(600)

let resizeObserver: ResizeObserver | null = null
onMounted(() => {
  if (containerRef.value) {
    containerWidth.value = containerRef.value.clientWidth
    resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) { containerWidth.value = entry.contentRect.width }
    })
    resizeObserver.observe(containerRef.value)
  }
})
onUnmounted(() => { resizeObserver?.disconnect() })

const cols = computed(() => Math.max(1, Math.floor(containerWidth.value / (props.minColumnWidth + props.gap))))
const cellWidth = computed(() => cols.value > 0 ? (containerWidth.value - (cols.value - 1) * props.gap) / cols.value : props.minColumnWidth)
const itemHeight = computed(() => {
  if (props.cardHeight > 0) return props.cardHeight
  return cellWidth.value + props.textAreaHeight
})

interface Row { __key: string; items: any[]; startIndex: number }
const rows = computed(() => {
  const c = cols.value
  if (c === 0) return []
  const result: Row[] = []
  for (let i = 0; i < props.items.length; i += c) {
    result.push({ __key: `row-${i}`, items: props.items.slice(i, i + c), startIndex: i })
  }
  return result
})
</script>

<template>
  <div ref="containerRef" class="w-full h-full">
    <RecycleScroller v-if="rows.length > 0" :items="rows" :item-size="itemHeight" key-field="__key" v-slot="{ item: row }"
      class="h-full">
      <div class="flex" :style="{ gap: gap + 'px', padding: '0 ' + (gap / 2) + 'px', marginBottom: gap + 'px' }">
        <div v-for="(item, i) in row.items" :key="i" :style="{ width: cellWidth + 'px', flexShrink: 0 }">
          <div v-if="squareItems" class="aspect-square">
            <slot :item="item" :index="row.startIndex + i" />
          </div>
          <div v-else>
            <slot :item="item" :index="row.startIndex + i" />
          </div>
        </div>
      </div>
    </RecycleScroller>
  </div>
</template>
