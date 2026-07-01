import type { ThemeColors } from './invoke'

export function hexToRgba(hex: string, alpha: number): string {
  const h = hex.replace('#', '')
  const r = parseInt(h.substring(0, 2), 16)
  const g = parseInt(h.substring(2, 4), 16)
  const b = parseInt(h.substring(4, 6), 16)
  return `rgba(${r}, ${g}, ${b}, ${alpha})`
}

function rgbToHex(r: number, g: number, b: number): string {
  return '#' + [r, g, b].map(v => Math.round(v).toString(16).padStart(2, '0')).join('')
}

export async function extractColorsFromArtwork(b64: string): Promise<ThemeColors | null> {
  return new Promise(resolve => {
    const img = new Image()
    img.onload = () => {
      try {
        const canvas = document.createElement('canvas')
        const size = 64
        canvas.width = size
        canvas.height = size
        const ctx = canvas.getContext('2d')
        if (!ctx) { resolve(null); return }
        ctx.drawImage(img, 0, 0, size, size)
        const data = ctx.getImageData(0, 0, size, size).data

        let rSum = 0, gSum = 0, bSum = 0, count = 0
        let maxSat = 0, vibrantR = 0, vibrantG = 0, vibrantB = 0
        let minLight = 1, mutedR = 0, mutedG = 0, mutedB = 0

        for (let i = 0; i < data.length; i += 4) {
          const r = data[i] / 255, g = data[i + 1] / 255, b = data[i + 2] / 255
          const max = Math.max(r, g, b), min = Math.min(r, g, b)
          const l = (max + min) / 2
          const d = max - min
          const s = d === 0 ? 0 : d / (1 - Math.abs(2 * l - 1))

          if (l > 0.1 && l < 0.9) {
            rSum += data[i]; gSum += data[i + 1]; bSum += data[i + 2]; count++
          }
          if (s > maxSat && l > 0.15 && l < 0.85) {
            maxSat = s
            vibrantR = data[i]; vibrantG = data[i + 1]; vibrantB = data[i + 2]
          }
          if (s > 0.1 && s < 0.5 && Math.abs(l - 0.5) < minLight) {
            minLight = Math.abs(l - 0.5)
            mutedR = data[i]; mutedG = data[i + 1]; mutedB = data[i + 2]
          }
        }

        if (count === 0) { resolve(null); return }
        resolve({
          vibrant: rgbToHex(vibrantR || rSum / count, vibrantG || gSum / count, vibrantB || bSum / count),
          muted: rgbToHex(mutedR || rSum / count, mutedG || gSum / count, mutedB || bSum / count),
          dominant: rgbToHex(rSum / count, gSum / count, bSum / count),
        })
      } catch {
        resolve(null)
      }
    }
    img.onerror = () => resolve(null)
    img.src = b64.startsWith('data:') ? b64 : `data:image/jpeg;base64,${b64}`
  })
}

export function applyThemeColors(colors: ThemeColors | null) {
  const root = document.documentElement
  if (!colors) {
    root.style.removeProperty('--dynamic-primary')
    root.style.removeProperty('--dynamic-surface')
    root.style.removeProperty('--dynamic-glow')
    return
  }
  const isDark = root.classList.contains('dark') || root.classList.contains('black')
  root.style.setProperty('--dynamic-primary', colors.vibrant)
  root.style.setProperty('--dynamic-surface', hexToRgba(colors.dominant, isDark ? 0.15 : 0.05))
  root.style.setProperty('--dynamic-glow', `0 0 40px ${hexToRgba(colors.vibrant, isDark ? 0.3 : 0.1)}`)
}
