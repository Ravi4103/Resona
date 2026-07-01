import { onMounted, onUnmounted } from 'vue'

export interface KeyboardShortcutOptions {
  ctrl?: boolean
  meta?: boolean
  shift?: boolean
  alt?: boolean
  key: string
}

export function useKeyboardShortcut(options: KeyboardShortcutOptions, callback: () => void) {
  const onKeyDown = (e: KeyboardEvent) => {
    if (
      e.key.toLowerCase() === options.key.toLowerCase() &&
      e.ctrlKey === !!options.ctrl &&
      e.metaKey === !!options.meta &&
      e.shiftKey === !!options.shift &&
      e.altKey === !!options.alt
    ) {
      e.preventDefault()
      callback()
    }
  }

  onMounted(() => { window.addEventListener('keydown', onKeyDown) })
  onUnmounted(() => { window.removeEventListener('keydown', onKeyDown) })
}
