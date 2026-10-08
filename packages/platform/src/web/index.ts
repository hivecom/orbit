import type { Platform } from "../types"
import { createAudioDevicePort } from "./audio-device"
import { createFileTransferPort } from "./file-transfer"
import { createIndexedDbCachePort } from "./history-cache"
import { createIrcPort } from "./irc"
import { createNotificationPort } from "./notification"
import { createTrayPort } from "./tray"

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
    irc: createIrcPort(),
  }
}
