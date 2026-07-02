import type { Platform } from "./types"

// TODO: this will consume tauri APIs imported from core

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
