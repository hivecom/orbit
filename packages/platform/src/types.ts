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
  setBadgeCount: (count: number) => Promise<void>
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
