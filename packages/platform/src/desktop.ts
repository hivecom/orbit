import type { Platform } from "./types"

export function createDesktopPlatform(): Platform {
  return {
    target: "desktop",
    notifications: null,
    tray: null,
    audioDevices: null,
    deepLinks: null,
    fileTransfer: null,
    dns: null,
  }
}
