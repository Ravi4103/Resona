import { defineStore } from 'pinia'
import { ref, computed, watch, shallowRef } from 'vue'
import { listen } from '@tauri-apps/api/event'
import * as api from '../lib/invoke'
import type { ThemeColors } from '../lib/invoke'
import { useAppStore } from './app'

export type PlayerMode = 'sticky' | 'mini' | 'fullscreen'

export const usePlayerStore = defineStore('player', () => {
  const currentTrack = ref<api.Track | null>(null)
  const queue = ref<api.Track[]>([])
  const queueIndex = ref(0)
  const isPlaying = ref(false)
  const isRadioPlaying = ref(false)
  const isQueueOpen = ref(false)
  const isLyricsOpen = ref(false)
  const isTrackInfoOpen = ref(false)
  const playerMode = ref<PlayerMode>('sticky')

  const position = ref(0)
  const duration = ref(0)
  const lyricsContent = ref<string | undefined>(undefined)
  const volume = ref(0.8)
  const shuffle = ref(false)
  const repeatMode = ref<'off' | 'all' | 'one'>('off')
  const theme = shallowRef<ThemeColors | null>(null)
  const artworkUrl = ref<string | null>(null)
  const artworkUrlSm = computed(() => artworkUrl.value)
  const artworkUrlMd = computed(() => artworkUrl.value)
  const isMuted = ref(false)
  const preMuteVolume = ref(0.8)
  const isPaused = computed(() => !isPlaying.value && currentTrack.value !== null)
  const isStopped = computed(() => !isPlaying.value && currentTrack.value === null)

  const appStore = useAppStore()
  const rawVolume = ref(0.8)

  function computeEffectiveVolume(): number {
    if (appStore.replaygainMode === 'off' || !currentTrack.value) {
      return rawVolume.value
    }
    const gain = appStore.replaygainMode === 'track'
      ? currentTrack.value.replaygain_track_gain
      : currentTrack.value.replaygain_album_gain
    if (gain === 0) return rawVolume.value
    return Math.min(1, Math.max(0, rawVolume.value * Math.pow(10, gain / 20)))
  }

  function applyVolume() {
    const eff = computeEffectiveVolume()
    volume.value = eff
    api.setVolume(eff).catch(() => {})
  }

  watch([() => currentTrack.value, () => appStore.replaygainMode], () => { applyVolume() })

  // Discord RPC
  const discordEnabled = ref(false)
  let discordStartTime = 0

  function enableDiscord(clientId: string) {
    discordEnabled.value = true
    discordStartTime = Date.now()
    api.discordConnect(clientId).catch(() => {})
  }

  function disableDiscord() {
    discordEnabled.value = false
    api.discordDisconnect().catch(() => {})
  }

  function updatePresence() {
    if (!discordEnabled.value || !currentTrack.value) return
    const track = currentTrack.value
    const stateStr = track.artist ? `Listening to ${track.artist}` : 'Listening'
    const details = track.title || 'Unknown Track'
    api.updateDiscordPresence(stateStr, details, Math.floor(discordStartTime / 1000), null).catch(() => {})
  }

  // Update Discord when track changes
  watch(currentTrack, () => { updatePresence() })

  const sleepTimerMinutes = ref(0) // 0 = off
  let sleepTimerInterval: ReturnType<typeof setInterval> | null = null
  let sleepTimerRemaining = ref(0)

  function setSleepTimer(minutes: number) {
    if (sleepTimerInterval) {
      clearInterval(sleepTimerInterval)
      sleepTimerInterval = null
    }
    sleepTimerMinutes.value = minutes
    if (minutes <= 0) {
      sleepTimerRemaining.value = 0
      return
    }
    sleepTimerRemaining.value = minutes * 60
    sleepTimerInterval = setInterval(() => {
      sleepTimerRemaining.value--
      if (sleepTimerRemaining.value <= 0) {
        if (sleepTimerInterval) clearInterval(sleepTimerInterval)
        sleepTimerInterval = null
        pause()
        sleepTimerMinutes.value = 0
        sleepTimerRemaining.value = 0
      }
    }, 1000)
  }

  function cancelSleepTimer() {
    if (sleepTimerInterval) {
      clearInterval(sleepTimerInterval)
      sleepTimerInterval = null
    }
    sleepTimerMinutes.value = 0
    sleepTimerRemaining.value = 0
  }

  const progressPercent = computed(() =>
    duration.value > 0 ? (position.value / duration.value) * 100 : 0,
  )

  // RAF interpolation state
  let lastSyncTime = performance.now()
  let lastSyncPosition = 0

  function syncInterpolation(pos: number) {
    lastSyncPosition = pos
    lastSyncTime = performance.now()
    position.value = lastSyncPosition
  }

  let _offFns: (() => void)[] = []
  let _initialized = false
  let _positionInterval: ReturnType<typeof setInterval> | null = null
  let _rafId: number | null = null

  function updateInterpolatedPosition() {
    if (isPlaying.value) {
      const now = performance.now()
      const elapsed = (now - lastSyncTime) / 1000
      position.value = Math.min(lastSyncPosition + elapsed, duration.value)
    } else {
      position.value = lastSyncPosition
    }
    _rafId = requestAnimationFrame(updateInterpolatedPosition)
  }

  async function init() {
    if (_initialized) return
    _initialized = true
    // Poll backend position occasionally to correct drift
    _positionInterval = setInterval(async () => {
      try {
        const pos = await api.getPosition()

        if (isPlaying.value) {
          syncInterpolation(pos)
        }
      } catch {}
    }, 2000)
    // Start RAF interpolation for smooth UI
    _rafId = requestAnimationFrame(updateInterpolatedPosition)

    let lastTrackId: string | null = null
    let lastTrackStart = 0

    _offFns = [
      await listen<string>('media-key', (event) => {
        if (event.payload === 'NextTrack') playNext()
        else if (event.payload === 'PreviousTrack') playPrev()
        else if (event.payload === 'Stop') { isPlaying.value = false; position.value = 0; api.togglePlay().catch(() => {}) }
      }),
      await listen('playback-ended', async () => {
        // Scrobble the track that finished
        if (lastTrackId && currentTrack.value) {
          const elapsed = (Date.now() - lastTrackStart) / 1000
          // Only scrobble if played more than 50% or 4 minutes
          if (elapsed > 240 || (duration.value > 0 && elapsed / duration.value > 0.5)) {
            api.scrobble(currentTrack.value.artist, currentTrack.value.title, currentTrack.value.album).catch(() => {})
          }
        }
        await playNext()
      }),
      await listen<string>('track-started', async (event) => {
        await api.recordPlay(event.payload).catch(() => {})
        lastTrackId = event.payload
        lastTrackStart = Date.now()
        // Update now-playing on Last.fm
        if (currentTrack.value) {
          api.nowPlaying(
            currentTrack.value.artist,
            currentTrack.value.title,
            currentTrack.value.album,
            Math.round(currentTrack.value.duration),
          ).catch(() => {})
        }
      }),
    ]
  }

  watch(currentTrack, async (track) => {
    if (!track) {
      artworkUrl.value = null
      theme.value = null
      lyricsContent.value = ''
      return
    }
    if (track.file_path) {
      try {
        let lrc: string | null = null
        if (appStore.preferLocalLyrics) {
          lrc = await api.getLyrics(track.file_path)
        }
        if (lrc) {
          lyricsContent.value = lrc
        } else {
          const row = await api.getTrackLyrics(track.id)
          lyricsContent.value = (row?.meta_content || row?.content) || ''
        }
      } catch {
        lyricsContent.value = ''
      }
    } else {
      lyricsContent.value = ''
    }

    try {
      const bytes = await api.getTrackArtwork(track.id, 'md')
      if (bytes && bytes.length > 0) {
        const uint8 = new Uint8Array(bytes)
        const blob = new Blob([uint8], { type: 'image/jpeg' })
        artworkUrl.value = URL.createObjectURL(blob)
        theme.value = await api.getTrackPalette(track.id)
      } else {
        artworkUrl.value = null
        theme.value = null
      }
    } catch {
      artworkUrl.value = null
      theme.value = null
    }
  })

  function dispose() {
    _offFns.forEach(fn => fn())
    _offFns = []
    if (_positionInterval) clearInterval(_positionInterval)
    if (_rafId !== null) cancelAnimationFrame(_rafId)
    _initialized = false
  }

  async function syncState() {
    try {
      const ps = await api.getPlayerState()
      volume.value = ps.volume
      position.value = ps.position
      lastSyncPosition = ps.position
      lastSyncTime = performance.now()
      isRadioPlaying.value = false
      if (ps.track_id) {
        const track = await api.getTrack(ps.track_id)
        if (track) {
          currentTrack.value = track
          duration.value = track.duration
          isPlaying.value = ps.is_playing
          return
        }
      }
      if (!ps.track_id || !(await api.getTrack(ps.track_id))) {
        isPlaying.value = false
      }
    } catch (e) {
      console.error('syncState failed:', e)
    }
  }

  async function play() {
    if (currentTrack.value) {
      isPlaying.value = true
      const wasPaused = await api.togglePlay()
      if (!wasPaused) {
        syncInterpolation(0)
        try {
          await api.playTrack(currentTrack.value.id)
        } catch (e) {
          isPlaying.value = false
          console.error('Playback failed:', e)
        }
      }
    }
  }

  async function pause() {
    isPlaying.value = false
    lastSyncPosition = position.value
    lastSyncTime = performance.now()
    await api.togglePlay()
  }

  async function togglePlayPause() {
    if (isPlaying.value) {
      await pause()
    } else if (currentTrack.value) {
      await play()
    }
  }

  async function playNext() {
    if (repeatMode.value === 'one') return
    if (shuffle.value) {
      queueIndex.value = Math.floor(Math.random() * queue.value.length)
    } else {
      queueIndex.value++
    }
    if (queueIndex.value >= queue.value.length) {
      if (repeatMode.value === 'all') {
        queueIndex.value = 0
      } else {
        isPlaying.value = false
        return
      }
    }
    const next = queue.value[queueIndex.value] ?? null
    currentTrack.value = next
    if (next) {
      duration.value = next.duration
      isPlaying.value = true
      syncInterpolation(0)
      await api.playTrack(next.id).catch(e => console.error('playNext failed', e))
    }
  }

  async function playPrev() {
    if (queueIndex.value > 0 && position.value > 3) {
      await seek(0)
      return
    }
    queueIndex.value--
    if (queueIndex.value < 0) {
      queueIndex.value = queue.value.length - 1
    }
    const prev = queue.value[queueIndex.value] ?? null
    currentTrack.value = prev
    if (prev) {
      duration.value = prev.duration
      isPlaying.value = true
      await api.playTrack(prev.id).catch(e => console.error('playPrev failed', e))
    }
  }

  async function seek(pos: number) {
    syncInterpolation(pos)
    await api.seek(pos)
  }

  async function setVolume(v: number) {
    rawVolume.value = v
    if (v > 0) isMuted.value = false
    if (appStore.replaygainMode === 'off') {
      volume.value = v
      await api.setVolume(v)
    } else {
      applyVolume()
    }
  }

  async function fastForward() {
    const newPos = Math.min(duration.value, position.value + 10)
    await seek(newPos)
  }

  async function rewind() {
    const newPos = Math.max(0, position.value - 10)
    await seek(newPos)
  }

  async function increaseVolume() {
    await setVolume(Math.min(1, rawVolume.value + 0.05))
  }

  async function decreaseVolume() {
    await setVolume(Math.max(0, rawVolume.value - 0.05))
  }

  async function toggleMute() {
    if (isMuted.value) {
      isMuted.value = false
      await setVolume(preMuteVolume.value || 0.8)
    } else {
      preMuteVolume.value = rawVolume.value
      isMuted.value = true
      volume.value = 0
      await api.setVolume(0)
    }
  }

  async function setMuted(m: boolean) {
    if (m !== isMuted.value) await toggleMute()
  }

  function reorderQueue(newQueue: api.Track[]) {
    queue.value = newQueue
  }

  async function setShuffle(s: boolean) {
    shuffle.value = s
    if (s && queue.value.length > 1) {
      // Reorder queue starting from current index
      const q = [...queue.value]
      const current = q[queueIndex.value]
      q.splice(queueIndex.value, 1)
      for (let i = q.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [q[i], q[j]] = [q[j], q[i]]
      }
      q.unshift(current)
      queue.value = q
      queueIndex.value = 0
    }
  }
  function cycleRepeat() {
    switch (repeatMode.value) {
      case 'off': repeatMode.value = 'all'; break
      case 'all': repeatMode.value = 'one'; break
      default: repeatMode.value = 'off'
    }
  }

  function toggleShuffle() {
    setShuffle(!shuffle.value)
  }

  const isMiniWindowOpen = ref(false)

  function cycleMode() {
    switch (playerMode.value) {
      case 'sticky':
        playerMode.value = 'mini'
        break
      case 'mini':
        playerMode.value = 'fullscreen'
        break
      default:
        playerMode.value = 'sticky'
    }
  }

  function setMode(mode: PlayerMode) {
    playerMode.value = mode
  }

  async function toggleMiniWindow() {
    if (isMiniWindowOpen.value) {
      const m = await import('../composables/useMiniPlayerWindow')
      await m.closeMiniWindow()
      isMiniWindowOpen.value = false
    } else {
      const m = await import('../composables/useMiniPlayerWindow')
      const ok = await m.openMiniWindow()
      isMiniWindowOpen.value = ok
    }
  }

  function next() {
    playNext()
  }

  function previous() {
    playPrev()
  }

  async function playTracks(tracks: api.Track[], startIndex: number) {
    queue.value = tracks
    queueIndex.value = startIndex
    const t = tracks[startIndex] ?? null
    currentTrack.value = t
    isPlaying.value = true
    if (t) {
      duration.value = t.duration
      syncInterpolation(0)
      try {
        await api.playTrack(t.id)
      } catch (e) {
        isPlaying.value = false
        console.error('Playback failed:', e)
      }
    }
  }

  function toggleQueue() {
    isQueueOpen.value = !isQueueOpen.value
    if (isQueueOpen.value) { isLyricsOpen.value = false; isTrackInfoOpen.value = false }
  }

  function toggleLyrics() {
    isLyricsOpen.value = !isLyricsOpen.value
    if (isLyricsOpen.value) { isQueueOpen.value = false; isTrackInfoOpen.value = false }
  }

  function toggleTrackInfoDrawer() {
    if (isTrackInfoOpen.value) {
      isTrackInfoOpen.value = false
    } else {
      openTrackInfo(currentTrack.value)
    }
  }

  const trackInfoTrack = ref<api.Track | null>(null)

  function openTrackInfo(track: api.Track | null) {
    if (!track) return
    trackInfoTrack.value = track
    isTrackInfoOpen.value = true
    isQueueOpen.value = false
    isLyricsOpen.value = false
  }

  function closeAllDrawers() {
    isQueueOpen.value = false
    isLyricsOpen.value = false
    isTrackInfoOpen.value = false
  }

  return {
    currentTrack, queue, queueIndex, isPlaying, isRadioPlaying, isPaused, isStopped,
    position, duration, volume, shuffle, repeatMode, playerMode,
    isQueueOpen, isLyricsOpen, isTrackInfoOpen, trackInfoTrack, theme,
    artworkUrl, artworkUrlSm, artworkUrlMd, isMuted,
    progressPercent, lyricsContent, init, dispose,
    play, pause, togglePlayPause, playNext, playPrev, syncState,
    seek, setVolume, setShuffle, cycleRepeat, playTracks, reorderQueue,
    toggleQueue, toggleLyrics, toggleTrackInfoDrawer, openTrackInfo, closeAllDrawers,
    toggleShuffle, cycleMode, setMode, isMiniWindowOpen, toggleMiniWindow,
    next, previous, fastForward, rewind, increaseVolume, decreaseVolume, toggleMute, setMuted,
    sleepTimerMinutes, sleepTimerRemaining, setSleepTimer, cancelSleepTimer,
    discordEnabled, enableDiscord, disableDiscord, updatePresence,
  }
})
