import { computed, type Ref } from 'vue'

export interface LyricLine {
  text: string
  time: number
}

const LRC_PATTERN = /^\[(\d+):(\d+\.\d+)\]/m
const LINE_PATTERN = /^\[(\d+):(\d+\.\d+)\](.*)/

function parseText(raw: string): { primary: string; secondary?: string } {
  const sep = raw.includes('^') ? '^' : raw.includes(' / ') ? ' / ' : null
  if (sep) {
    const parts = raw.split(sep).map(s => s.trim())
    return { primary: parts[0], secondary: parts[1] }
  }
  return { primary: raw }
}

export function useLyrics(lyrics: Ref<string | undefined>) {
  const isSynced = computed(() => !!lyrics.value && LRC_PATTERN.test(lyrics.value))

  const syncedLines = computed<LyricLine[]>(() => {
    if (!lyrics.value) return []
    return lyrics.value.split('\n').flatMap(line => {
      const match = line.match(LINE_PATTERN)
      if (!match) return []
      const minutes = parseInt(match[1], 10)
      const seconds = parseFloat(match[2])
      const raw = match[3].trim()
      const { primary, secondary } = parseText(raw)
      return [{ text: primary, secondary, time: minutes * 60 + seconds }]
    })
  })

  const plainLines = computed<{ primary: string; secondary?: string }[]>(() => {
    if (!lyrics.value) return []
    return lyrics.value
      .split('\n')
      .map(l => l.replace(/^\[(\d+):(\d+\.\d+)\]/, '').trim())
      .filter(l => l)
      .map(l => parseText(l))
  })

  return { isSynced, syncedLines, plainLines }
}
