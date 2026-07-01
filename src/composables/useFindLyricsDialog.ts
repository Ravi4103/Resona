import { ref } from 'vue'

export function useFindLyricsDialog() {
  const open = ref(false)

  function show() { open.value = true }
  function hide() { open.value = false }

  return { open, show, hide }
}
