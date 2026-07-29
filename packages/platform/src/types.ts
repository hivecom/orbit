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
  /**
   * Sets the red badge count on the favicon/app icon. Setting 0 removes the badge.
   */
  setBadgeCount: (count: number) => Promise<void>
  /**
   * Sets the document title or application tray on-hover title
   */
  setTitle: (title: string) => Promise<void>
}

export interface FaviconData {
  element: HTMLLinkElement
  cleanBitmap: ImageBitmap
  url: string
}

export interface AudioDevice {
  id: string
  label: string
  kind: "input" | "output"
}

export interface AudioDevicePort {
  enumerate: () => Promise<AudioDevice[]>
  onChange: (listener: () => void) => () => void
}

export interface DeepLinkPort {
  onOpen: (listener: (url: string) => void) => () => void
}

export interface FileDownloadRequest {
  url: string
  filename: string
}

export interface FileTransferPort {
  download: (request: FileDownloadRequest) => Promise<void>
}

export interface SrvRecord {
  target: string
  port: number
  priority: number
  weight: number
}

export interface DnsPort {
  resolveSrv: (service: string) => Promise<SrvRecord[]>
}

export interface Platform {
  readonly target: "web" | "desktop" | "mobile"
  readonly notifications: NotificationPort | null
  readonly tray: TrayPort | null
  readonly audioDevices: AudioDevicePort | null
  readonly deepLinks: DeepLinkPort | null
  readonly fileTransfer: FileTransferPort | null
  readonly dns: DnsPort | null
}
