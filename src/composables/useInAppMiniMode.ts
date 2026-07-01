import { appWindow, LogicalSize } from '@tauri-apps/api/window'

let prevSize: { width: number; height: number } | null = null
let prevAlwaysOnTop: boolean | null = null
let inMini = false

export async function enterMiniMode(width = 360, height = 88, makeAlwaysOnTop = false) {
  if (inMini) return
  inMini = true
  try {
    // store previous size
    const size = await appWindow.outerSize()
    prevSize = { width: size.width, height: size.height }
  } catch (e) {
    // ignore
    prevSize = null
  }

  try {
    // store previous always-on-top state if available (best-effort)
    try {
      // There's no getter in some versions; assume false and restore to false
      prevAlwaysOnTop = false
    } catch {}

    // Attempt to remove decorations for a compact look (best-effort)
    try {
      await appWindow.setDecorations(false)
    } catch {}

    try {
      await appWindow.setSize(new LogicalSize(width, height))
    } catch (e) {
      console.warn('[in-app-mini] setSize failed', e)
    }

    if (makeAlwaysOnTop) {
      try {
        await appWindow.setAlwaysOnTop(true)
      } catch (e) {
        console.warn('[in-app-mini] setAlwaysOnTop failed', e)
      }
    }
  } catch (e) {
    console.warn('[in-app-mini] enterMiniMode failed', e)
  }
}

export async function exitMiniMode() {
  if (!inMini) return
  inMini = false
  try {
    if (prevSize) {
      try {
        await appWindow.setSize(new LogicalSize(prevSize.width, prevSize.height))
      } catch (e) {
        console.warn('[in-app-mini] restore size failed', e)
      }
    }

    try {
      await appWindow.setDecorations(true)
    } catch (e) {
      // ignore
    }

    if (prevAlwaysOnTop) {
      try {
        await appWindow.setAlwaysOnTop(prevAlwaysOnTop)
      } catch (e) {
        console.warn('[in-app-mini] restore alwaysOnTop failed', e)
      }
    } else {
      try {
        await appWindow.setAlwaysOnTop(false)
      } catch {}
    }
  } catch (e) {
    console.warn('[in-app-mini] exitMiniMode failed', e)
  }
}
