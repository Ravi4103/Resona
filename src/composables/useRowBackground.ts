import type { Ref } from 'vue'

export function useRowBackground(variant?: Ref<'default' | 'glass' | undefined>) {
  function rowBg(index: number, opaque = false): string {
    if (variant?.value === 'glass' && opaque) return 'transparent'
    if (index % 2 !== 0) return opaque ? 'transparent' : 'transparent'
    return opaque ? 'transparent' : 'var(--bg-zebra)'
  }
  return { rowBg }
}
