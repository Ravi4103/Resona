import { defineStore } from 'pinia'
import { ref } from 'vue'
import { darken, hexToRgba } from '../lib/utils'

export const colorDefaults = {
  '--primary': '#e11d48',
  '--favorite': '#f43f5e',
  '--success': '#22c55e',
  '--warning': '#f59e0b',
  '--error': '#dc2626',
  '--info': '#0ea5e9',
  '--accent-secondary': '#06b6d4',
  '--now-playing': '',
} as const

export type ColorKey = keyof typeof colorDefaults

export const useAppStore = defineStore('app', () => {
  const theme = ref<'system' | 'light' | 'dark' | 'black'>('dark')
  const startAtLogin = ref(false)
  const showTrayIcon = ref(true)
  const lastfmUsername = ref('')
  const replaygainMode = ref<'off' | 'track' | 'album'>('off')
  const eqEnabled = ref(true)
  const enableLrclib = ref(true)
  const enableKugou = ref(true)
  const preferLocalLyrics = ref(true)
  const autoFetchLyrics = ref(false)
  const useOnlineArtistArtwork = ref(true)
  const preventSleepWhilePlaying = ref(false)
  const showPlayerIndicator = ref(true)
  const remoteServerEnabled = ref(false)
  const remoteServerPort = ref(8888)
  const remoteServerPassword = ref('0000')
  const discordClientId = ref('')
  const discordEnabled = ref(false)

  const sidebarWidth = ref(parseInt(localStorage.getItem('resona-sidebar-width') || '260', 10))

  const activeSettingsTab = ref<'general' | 'library' | 'integrations' | 'playback' | 'remote' | 'about'>('general')

  const colors = ref<Record<ColorKey, string>>({ ...colorDefaults })

  function applyColor(key: ColorKey, hex: string) {
    const root = document.documentElement
    if (key === '--primary') {
      root.style.setProperty('--primary', hex)
      root.style.setProperty('--primary-hover', darken(hex, 0.85))
      root.style.setProperty('--primary-muted', hexToRgba(hex, 0.1))
      root.style.setProperty('--primary-glow', hexToRgba(hex, 0.15))
    } else if (key === '--now-playing' && !hex) {
      root.style.removeProperty('--now-playing')
    } else {
      root.style.setProperty(key, hex || null)
    }
    colors.value[key] = hex
    localStorage.setItem(`resona-color-${key}`, hex)
  }

  function loadSavedColors() {
    for (const key of Object.keys(colorDefaults) as ColorKey[]) {
      const saved = localStorage.getItem(`resona-color-${key}`)
      if (saved !== null && (key !== '--now-playing' || saved)) {
        applyColor(key, saved)
      }
    }
  }

  function resetColors() {
    for (const [key, val] of Object.entries(colorDefaults) as [ColorKey, string][]) {
      applyColor(key, val)
    }
  }

  function applyTheme(newTheme: 'system' | 'light' | 'dark' | 'black') {
    const root = document.documentElement
    root.classList.remove('light', 'dark', 'black')
    if (newTheme === 'system') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      root.classList.add(prefersDark ? 'dark' : 'light')
    } else {
      root.classList.add(newTheme)
    }
    theme.value = newTheme
    localStorage.setItem('resona-theme', newTheme)
  }

  function loadSavedTheme() {
    const saved = localStorage.getItem('resona-theme') as typeof theme.value | null
    if (saved) applyTheme(saved)
  }

  function updateSidebarWidth(width: number) {
    sidebarWidth.value = Math.max(180, Math.min(400, width))
    localStorage.setItem('resona-sidebar-width', String(sidebarWidth.value))
  }

  function setActiveSettingsTab(tab: typeof activeSettingsTab.value) {
    activeSettingsTab.value = tab
  }

  function updateTheme(newTheme: typeof theme.value) { applyTheme(newTheme) }
  async function updateStartAtLogin(val: boolean) {
    startAtLogin.value = val
    try {
      if (val) {
        const { enable } = await import('@tauri-apps/plugin-autostart')
        await enable()
      } else {
        const { disable } = await import('@tauri-apps/plugin-autostart')
        await disable()
      }
    } catch (e) {
      console.error('Autostart toggle failed', e)
    }
  }
  function updateShowTrayIcon(val: boolean) { showTrayIcon.value = val }
  function updateLastFmUsername(val: string) {
    lastfmUsername.value = val
    localStorage.setItem('resona-lastfm-user', val)
  }

  function updateDiscordClientId(val: string) {
    discordClientId.value = val
    localStorage.setItem('resona-discord-id', val)
  }

  function loadSavedSettings() {
    const lastfm = localStorage.getItem('resona-lastfm-user')
    if (lastfm) lastfmUsername.value = lastfm
    const discordId = localStorage.getItem('resona-discord-id')
    if (discordId) discordClientId.value = discordId
  }
  function updateReplaygainMode(val: 'off' | 'track' | 'album') { replaygainMode.value = val }
  function updateEqEnabled(val: boolean) { eqEnabled.value = val }
  function updateEnableLrclib(val: boolean) { enableLrclib.value = val }
  function updateEnableKugou(val: boolean) { enableKugou.value = val }
  function updatePreferLocalLyrics(val: boolean) { preferLocalLyrics.value = val }
  function updateAutoFetchLyrics(val: boolean) { autoFetchLyrics.value = val }
  function updateUseOnlineArtistArtwork(val: boolean) { useOnlineArtistArtwork.value = val }
  async function updatePreventSleepWhilePlaying(val: boolean) {
    preventSleepWhilePlaying.value = val
    try {
      if (val) {
        const { preventSleep } = await import('../lib/invoke')
        await preventSleep()
      } else {
        const { allowSleep } = await import('../lib/invoke')
        await allowSleep()
      }
    } catch (e) {
      console.error('Sleep toggle failed', e)
    }
  }
  function updateShowPlayerIndicator(val: boolean) { showPlayerIndicator.value = val }
  async function updateRemoteServerEnabled(val: boolean) {
    remoteServerEnabled.value = val
    try {
      if (val) {
        const { remoteStart } = await import('../lib/invoke')
        await remoteStart(remoteServerPort.value, remoteServerPassword.value)
      } else {
        const { remoteStop } = await import('../lib/invoke')
        await remoteStop()
      }
    } catch (e) {
      console.error('Remote server toggle failed', e)
    }
  }
  function updateRemoteServerPort(val: number) { remoteServerPort.value = val }
  function updateRemoteServerPassword(val: string) { remoteServerPassword.value = val }

  return {
    theme, startAtLogin, showTrayIcon,
    lastfmUsername, replaygainMode, eqEnabled, enableLrclib, enableKugou, preferLocalLyrics, autoFetchLyrics,
    useOnlineArtistArtwork, preventSleepWhilePlaying, showPlayerIndicator,
    remoteServerEnabled, remoteServerPort, remoteServerPassword,
    discordClientId, discordEnabled,
    sidebarWidth, activeSettingsTab,
    colors,
    applyTheme, loadSavedTheme, loadSavedSettings, updateSidebarWidth, setActiveSettingsTab,
    updateTheme, updateStartAtLogin, updateShowTrayIcon,
    updateLastFmUsername, updateDiscordClientId, updateReplaygainMode, updateEqEnabled, updateEnableLrclib, updateEnableKugou,
    updatePreferLocalLyrics, updateAutoFetchLyrics, updateUseOnlineArtistArtwork,
    updatePreventSleepWhilePlaying, updateShowPlayerIndicator,
    updateRemoteServerEnabled, updateRemoteServerPort, updateRemoteServerPassword,
    applyColor, loadSavedColors, resetColors,
  }
})
