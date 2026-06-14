import type { AudioDevice, AudioDevicePort, FileTransferPort, NotificationPort, Platform } from "./types"

function createNotificationPort(): NotificationPort {
  return {
    async requestPermission() {
      if (!("Notification" in globalThis)) return false
      if (Notification.permission === "granted") return true
      if (Notification.permission === "denied") return false
      const result = await Notification.requestPermission()
      return result === "granted"
    },
    async notify({ title, body, icon }) {
      if (!("Notification" in globalThis) || Notification.permission !== "granted") return
      // eslint-disable-next-line no-new -- the Notification side effect is the point
      new Notification(title, { body, icon })
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

// Browser platform adapter. Capabilities that require a native shell - the
// system tray, orbit:// deep links, and DNS SRV resolution - are null; core
// degrades gracefully. A server-side resolver endpoint covers DNS on the web.
export function createWebPlatform(): Platform {
  return {
    target: "web",
    notifications: createNotificationPort(),
    tray: null,
    audioDevices: createAudioDevicePort(),
    deepLinks: null,
    fileTransfer: createFileTransferPort(),
    dns: null,
  }
}
