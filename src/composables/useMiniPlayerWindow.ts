import { getCurrentWebviewWindow, WebviewWindow } from '@tauri-apps/api/webviewWindow'
import type { WindowOptions } from '@tauri-apps/api/window'

const MINI_WINDOW_LABEL = 'mini-player'
let miniWindow: WebviewWindow | null = null

export async function openMiniWindow(): Promise<boolean> {
  if (miniWindow) {
    try {
      await miniWindow.show()
      await miniWindow.setFocus()
      return true
    } catch {
      miniWindow = null
    }
  }

  const baseUrl = window.location.origin + window.location.pathname.replace(/\/$/, '')
  const savedOnTop = localStorage.getItem('resona-mini-always-on-top') === 'true'

  const winOptions: WindowOptions = {
    title: 'Reson\u00e1 Mini Player',
    width: 300,
    height: 300,
    minWidth: 280,
    minHeight: 180,
    maxWidth: 500,
    maxHeight: 500,
    resizable: true,
    decorations: false,
    alwaysOnTop: savedOnTop,
    skipTaskbar: true,
  }

  try {
    const mainWin = getCurrentWebviewWindow()
    const pos = await mainWin.outerPosition()
    const size = await mainWin.outerSize()
    const x = Math.round(pos.x + size.width - 320)
    const y = Math.round(pos.y + size.height - 340)

    miniWindow = new WebviewWindow(MINI_WINDOW_LABEL, {
      url: baseUrl + '/#/mini-player',
      x,
      y,
      ...winOptions,
    })

    miniWindow.once('tauri://error', (e) => {
      console.error('[miniWindow] error', e)
      miniWindow = null
    })

    miniWindow.onCloseRequested(() => {
      miniWindow = null
    })

    return true
  } catch (e) {
    console.error('[miniWindow] failed to create', e)
    return false
  }
}

export async function closeMiniWindow() {
  if (miniWindow) {
    try {
      await miniWindow.close()
    } catch {}
    miniWindow = null
  }
}

export async function toggleMiniWindow() {
  if (miniWindow) {
    await closeMiniWindow()
  } else {
    await openMiniWindow()
  }
}