export interface NotificationOptions {
  title: string
  body?: string
  icon?: string
}

export interface NotificationPort {
  requestPermission: () => Promise<boolean>
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
  readonly target: "web" | "desktop" | "mobile"
  readonly notifications: NotificationPort
  readonly tray: TrayPort | null
  readonly audioDevices: AudioDevicePort
  readonly deepLinks: DeepLinkPort | null
  readonly fileTransfer: FileTransferPort
  readonly dns: DnsPort | null
}
