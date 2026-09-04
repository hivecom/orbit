import { createMockPlatform } from "./mock"
import { TaskQueue } from "./task"
import type { AudioDevice, AudioDevicePort, FileTransferPort, NotificationPort, Platform, TrayPort } from "./types"

function createNotificationPort(): NotificationPort {
  return {
    async requestPermission() {
      if (!("Notification" in globalThis)) return false
      if (Notification.permission === "granted") return true
      if (Notification.permission === "denied") return false
      const result = await Notification.requestPermission()
      return result === "granted"
    },
    notify({ title, body, icon }) {
      if (!("Notification" in globalThis) || Notification.permission !== "granted") {
        return null
      }

      void new Notification(title, { body, icon })
    },
  }
}

function createAudioDevicePort(): AudioDevicePort {
  return {
    async enumerate() {
      if (!navigator.mediaDevices?.enumerateDevices) return []
      const devices = await navigator.mediaDevices.enumerateDevices()
      return devices
        .filter((device) => device.kind === "audioinput" || device.kind === "audiooutput")
        .map<AudioDevice>((device) => ({
          id: device.deviceId,
          label: device.label || "Unknown device",
          kind: device.kind === "audioinput" ? "input" : "output",
        }))
    },
    onChange(listener) {
      const target = navigator.mediaDevices
      if (!target) return () => {}
      target.addEventListener("devicechange", listener)
      return () => target.removeEventListener("devicechange", listener)
    },
  }
}

function createFileTransferPort(): FileTransferPort {
  return {
    async download({ url, filename }) {
      const anchor = document.createElement("a")
      anchor.href = url
      anchor.download = filename
      anchor.rel = "noopener"
      document.body.append(anchor)
      anchor.click()
      anchor.remove()
    },
  }
}

export interface FaviconData {
  element: HTMLLinkElement
  cleanBitmap: ImageBitmap
  url: string
}

function createTrayPort(): TrayPort {
  // Store reference to the original favicon, in case the badge is ever cleared
  let favicon: FaviconData

  // Store favicon image size for badge positioning
  let size = 0

  const canvas = document.createElement("canvas")
  const ctx = canvas.getContext("2d")

  // Implemented with the help of https://stackoverflow.com/a/65720799
  const BADGE_SIZE = 0.6
  const BADGE_TEXT_COLOR = "rgb(255, 255, 255)"

  const getBadgeBgcolor = () => window.getComputedStyle(document.body).getPropertyValue("--color-text-red")

  const tasks = new TaskQueue()

  // Returns the favicon object based on the initial favicon.
  async function getCurrentFavicon(): Promise<FaviconData | null> {
    const element = document.querySelector<HTMLLinkElement>('head link[rel*="icon"]')
    if (!element) return null
    const url = element?.href ?? `${location.origin}/favicon.ico`

    try {
      const res = await fetch(url)

      if (!res.ok) return null

      const faviconBlob = await res.blob()
      const cleanBitmap = await createImageBitmap(faviconBlob)

      return {
        element,
        cleanBitmap,
        url,
      }
    } catch {
      return null
    }
  }

  // Draws a badge with a count on top of favicon
  function drawBadgeCount(count: number) {
    if (!ctx) return

    resetFavicon()

    if (count > 0) {
      const badgeSize = size * BADGE_SIZE
      const radius = badgeSize / 3

      const xa = size - badgeSize
      const ya = 0

      ctx.beginPath()
      ctx.roundRect(xa, ya, badgeSize, badgeSize, radius)
      ctx.fillStyle = getBadgeBgcolor()
      ctx.fill()

      ctx.textAlign = "center"
      ctx.textBaseline = "middle"
      ctx.font = `bold ${badgeSize * 0.75}px Arial`
      ctx.fillStyle = BADGE_TEXT_COLOR

      const text = Math.min(count, 99).toString()
      ctx.fillText(text, xa + badgeSize / 2, ya + badgeSize / 2 + 0.5)
    }

    // Update attribute
    favicon.element.setAttribute("href", canvas.toDataURL())
  }

  // Draws a red circle badge on top of a favicon
  function drawBadgeAlert() {
    if (!ctx) return

    resetFavicon()

    const arcSize = size * 0.22
    const x = size - arcSize + 1
    const y = arcSize

    ctx.beginPath()
    ctx.arc(x, y, arcSize, 0, 2 * Math.PI)
    ctx.fillStyle = getBadgeBgcolor()
    ctx.fill()

    favicon.element.setAttribute("href", canvas.toDataURL())
  }

  // Resets favicon back to the original state
  function resetFavicon(apply?: boolean) {
    if (!ctx) return

    ctx.clearRect(0, 0, size, size)
    ctx.drawImage(favicon.cleanBitmap, 0, 0, size, size)

    // If true, reset is immediately applied
    if (apply) {
      favicon.element.setAttribute("href", canvas.toDataURL())
    }
  }

  // Initialize the canvas. Fetch the clean favicon and store it
  async function initializeCanvas() {
    if (favicon) return

    const res = await getCurrentFavicon()
    if (!res) {
      throw new TypeError("Could not initialize favicon")
    }

    favicon = res
    const img = document.createElement("img")
    img.src = favicon.url

    // Get the natural size. We don't expect to encounter errors at this
    // stage, because if we get here, a successful load of the favicon file
    // has already occurred
    size = await new Promise<number>((resolve, reject) => {
      img.onload = (event) => {
        resolve((event.target as HTMLImageElement).naturalWidth)
      }
      img.onerror = () => reject(new Error("Failed to load favicon bitmap data"))
    })

    canvas.width = size
    canvas.height = size
  }

  // Methods are async, because tauri might perform actual file oprations,
  // however on web these are just snake oil. Promise resolves immediately
  return {
    async setTitle(title) {
      document.title = title
    },
    async setBadgeCount(count) {
      void tasks.queue(async () => {
        await initializeCanvas()

        if (count > 0) {
          drawBadgeCount(count)
        } else {
          void this.removeBadge()
        }
      })
    },
    async addBadgeAlert() {
      void tasks.queue(async () => {
        await initializeCanvas()
        drawBadgeAlert()
      })
    },
    async removeBadge() {
      // Ignore if favicon is not initialized = badge hasn't been used and does
      // not need to be reset
      if (!favicon) return

      resetFavicon(true)
    },
  }
}

// TODO: Implement
function createIndexedDbCachePort() {
  return createMockPlatform("web").historyCache
}

// Browser platform adapter. Capabilities that require a native shell - the
// system tray, orbit:// deep links, and DNS SRV resolution - are null
export function createWebPlatform(): Platform {
  return {
    target: "web",
    notifications: createNotificationPort(),
    tray: createTrayPort(),
    audioDevices: createAudioDevicePort(),
    deepLinks: null,
    fileTransfer: createFileTransferPort(),
    dns: null,
    historyCache: createIndexedDbCachePort(),
  }
}
