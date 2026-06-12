import type { App, InjectionKey } from "vue"
import { inject } from "vue"

// The platform contract. `packages/core` owns this and consumes it through
// the injected `Platform`; it never imports `@tauri-apps/api` or touches raw
// `navigator.*` directly. Each app entrypoint (web, desktop, mobile) supplies
// a concrete adapter from `packages/platform` and injects it once at boot.
//
// The seam is shaped by capability, not by platform: core asks a port to do a
// thing, it never asks "am I running in Tauri". A port an environment cannot
// provide is `null`, and core degrades explicitly rather than scattering
// platform checks.

export interface NotificationOptions {
  title: string
  body?: string
  icon?: string
}

export interface NotificationPort {
  /** Request permission to display notifications. Resolves to whether granted. */
  requestPermission: () => Promise<boolean>
  /** Display a notification. No-op if permission has not been granted. */
  notify: (options: NotificationOptions) => Promise<void>
}

export interface TrayPort {
  /** Set the unread badge count shown on the tray / dock / taskbar icon. */
  setBadgeCount: (count: number) => Promise<void>
}

export interface AudioDevice {
  id: string
  label: string
  kind: "input" | "output"
}

export interface AudioDevicePort {
  enumerate: () => Promise<AudioDevice[]>
  /** Subscribe to device changes. Returns an unsubscribe function. */
  onChange: (listener: () => void) => () => void
}

export interface DeepLinkPort {
  /** Subscribe to inbound orbit:// / satellite:// links. Returns unsubscribe. */
  onOpen: (listener: (url: string) => void) => () => void
}

export interface FileDownloadRequest {
  url: string
  filename: string
}

export interface FileTransferPort {
  /** Download a remote file to the user's machine. */
  download: (request: FileDownloadRequest) => Promise<void>
}

export interface SrvRecord {
  target: string
  port: number
  priority: number
  weight: number
}

export interface DnsPort {
  /** Resolve a DNS SRV record (e.g. `_satellite._tcp.example.org`). */
  resolveSrv: (service: string) => Promise<SrvRecord[]>
}

export interface Platform {
  /** Identifies which adapter is active. */
  readonly target: "web" | "desktop" | "mobile"
  readonly notifications: NotificationPort
  readonly tray: TrayPort | null
  readonly audioDevices: AudioDevicePort
  readonly deepLinks: DeepLinkPort | null
  readonly fileTransfer: FileTransferPort
  readonly dns: DnsPort | null
}

export const PLATFORM_KEY: InjectionKey<Platform> = Symbol("orbit-platform")

/** Wire a concrete platform adapter into the app at boot. */
export function providePlatform(app: App, platform: Platform): void {
  app.provide(PLATFORM_KEY, platform)
}

/** Access the injected platform adapter from a component or composable. */
export function usePlatform(): Platform {
  const platform = inject(PLATFORM_KEY)
  if (!platform) {
    throw new Error("No platform adapter provided. Call providePlatform(app, ...) at app boot.")
  }
  return platform
}
