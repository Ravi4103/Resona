import { onMounted, type Ref } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'

export function useRestoreScroll(scrollContainer: Ref<HTMLElement | null>, key: string) {
  const storageKey = `scroll-pos:${key}`

  onMounted(() => {
    const saved = sessionStorage.getItem(storageKey)
    if (saved && scrollContainer.value) {
      setTimeout(() => {
        if (scrollContainer.value) {
          scrollContainer.value.scrollTop = parseInt(saved, 10)
        }
      }, 50)
    }
  })

  onBeforeRouteLeave(() => {
    if (scrollContainer.value) {
      sessionStorage.setItem(storageKey, String(scrollContainer.value.scrollTop))
    }
  })
}
