import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useDeviceStore = defineStore('device', () => {
  const isMac = ref(false)
  const isWindows = ref(true)
  const isLinux = ref(false)

  function init() {
    const platform = navigator.platform.toLowerCase()
    isMac.value = platform.includes('mac')
    isWindows.value = platform.includes('win')
    isLinux.value = platform.includes('linux')
  }

  return { isMac, isWindows, isLinux, init }
})
