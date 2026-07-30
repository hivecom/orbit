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
   * Sets the red badge count on the favicon/app icon. Count must be grater than 0
   */
  setBadgeCount: (count: number) => Promise<void>
  /**
   * Add/Remove an alert circle from the document/tray icon.
   */
  addBadgeAlert: () => Promise<void>
  /**
   * Remove any active badge from the icon
   */
  removeBadge: () => Promise<void>
  /**
   * Sets the application title.
   *
   * - on web it's the document.title
   * - on native it's the title of the tooltip which appears on hover over the app icon
   * - on mobile this is a noop
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
  readonly notifications: NotificationPort
  readonly tray: TrayPort
  readonly audioDevices: AudioDevicePort
  readonly deepLinks: DeepLinkPort | null
  readonly fileTransfer: FileTransferPort
  readonly dns: DnsPort | null
}
