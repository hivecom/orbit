export interface NotificationOptions {
  title: string
  body?: string
  icon?: string
}

export interface NotificationPort {
  requestPermission: () => Promise<boolean>
  // TODO: when developing platforms outside of web, we might want to dismiss a
  // notification from code. For that we'll need some kind of a reference, so
  // this method might have to return an id so a new `dismiss(id)` method can
  // call it
  notify: (options: NotificationOptions) => void
}

export interface TrayPort {
  /**
   * Sets the red badge count on the favicon/app icon. Passing 0 clears the badge
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

interface CachedMessage {
  msgid: string // primary key: server msgid, or a synthetic evt:* key for keyless lines
  target: string // channel or DM this belongs to
  serverTime: number // sort key, from server-time (epoch ms)
  account: string | null // server-asserted author identity (account-tag), null for unauthenticated
  nick: string // nick at send time (display only; account is authoritative)
  type: "privmsg" | "notice" | "action" | "join" | "part"
  text: string
  tags: Record<string, string> // surviving +orbit/* and +draft/* tags (reply ref, reactions, etc.)
  redacted?: boolean // tombstone overlay; original text is NOT retained when set
  edited?: boolean // set when text was edited in place via the interim +orbit/msg-amend tag
}

export interface HistoryCachePort {
  seed(target: string, limit: number): Promise<CachedMessage[]>
  pageBefore(target: string, beforeMsgid: string, limit: number): Promise<CachedMessage[]>
  pageAfter(target: string, afterMsgid: string, limit: number): Promise<CachedMessage[]>
  upsert(messages: CachedMessage[]): Promise<void> // batched, dedupes by msgid
  markRedacted(msgid: string): Promise<void> // tombstone overlay
  bufferStats(): Promise<BufferStats[]> // Storage management surface
  prune(target: string, keepCount: number): Promise<void>
  export(target: string): Promise<CachedMessage[]>
  clear(): Promise<void> // wipe this account's cache
}

export interface BufferStats {
  target: string
  count: number
  estimatedBytes: number
  oldest: number // server-time epoch ms
  newest: number
}

export interface Platform {
  readonly target: "web" | "desktop" | "mobile"
  /**
   * @targets Web, Desktop, Mobile
   */
  readonly notifications: NotificationPort
  /**
   * @targets Web, Desktop, Mobile
   */
  readonly tray: TrayPort
  /**
   * @targets Web, Desktop, Mobile
   */
  readonly audioDevices: AudioDevicePort
  /**
   * @targets Desktop, Mobile
   */
  readonly deepLinks: DeepLinkPort | null
  /**
   * @targets Web, Desktop, Mobile
   */
  readonly fileTransfer: FileTransferPort
  /**
   * @targets Desktop, Mobile
   */
  readonly dns: DnsPort | null
  /**
   * @targets Web, Desktop, Mobile
   */
  readonly historyCache: HistoryCachePort
}
