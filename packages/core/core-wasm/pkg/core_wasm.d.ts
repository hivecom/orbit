/* tslint:disable */
/* eslint-disable */
export interface Capabilities {
  message_tags: Capability
  message_redaction: Capability
  message_edit: Capability
  multiline: Capability
  metadata: Capability
  webpush: Capability
  echo_messages: Capability
  sasl: Capability
  history: Capability
  event_playback: Capability
  account_registration: Capability
  server_time: Capability
  account_notify: Capability
  account_tag: Capability
  away_notify: Capability
  batch: Capability
  cap_notify: Capability
  chghost: Capability
  channel_rename: Capability
  extended_isupport: Capability
  languages: Capability
  no_implicit_names: Capability
  persistence: Capability
  pre_away: Capability
  read_marker: Capability
  relaymsg: Capability
  extended_join: Capability
  extended_monitor: Capability
  invite_notify: Capability
  labeled_response: Capability
  multi_prefix: Capability
  setname: Capability
  standard_replies: Capability
  tls: Capability
  userhost_in_names: Capability
}

export interface Capability {
  has: boolean
  enabled: boolean
}

export interface Channel {
  metadata: ChannelMetadata
  messages: Message[]
  users: ChannelUser[]
}

export interface ChannelInfo {
  name: string
  user_count: number
  topic: string
}

export interface ChannelMetadata {
  name: string
  display_name: string | null
  topic: string | null
  description: string | null
  icon: string | null
}

export interface ChannelUser {
  nickname: string
  role: ChannelRole
}

export interface History {
  target: string
  messages: Message[]
}

export interface Message {
  text: TextMessage | null
  metadata: MessageMetadata
}

export interface MessageMetadata {
  msgid: string
  server_time: number
  message_type: MessageType
  user: string
}

export interface MessageReference {
  /**
   * Unset if message wasn't found or if reply wasn't to a text message
   */
  text: string | null
  /**
   * Unset if message wasn't found
   */
  username: string | null
}

export interface OrbitError {
  kind: OrbitErrorKind
  description: string
}

export interface Server {
  id: number
  metadata: ServerMetadata
  channels: Record<string, Channel>
  capabilities: Capabilities
  support: Support
  users: Record<string, User>
  me: User | null
}

export interface ServerMetadata {
  name: string | null
  motd: string | null
  address: string
  transponder_url: string | null
  satellite_url: string | null
  depot_url: string | null
}

export interface Support {
  accept: number | null
  account_extended_ban: string[] | null
  away_length: number | null
  bot: string | null
  caller_id: string | null
  case_mapping: string | null
  channel_limit: Record<string, number> | null
  channel_modes: string[] | null
  channel_length: number | null
  channel_types: string | null
  chat_history: number | null
  client_tag_deny: string[] | null
  deaf: string | null
  elist: string | null
  esilence: string | null
  etrace: boolean
  excepts: boolean
  extban: string | null
  host_length: number | null
  fnc: boolean
  forward: string | null
  invex: boolean
  key_length: number | null
  knock: boolean
  kick_length: number | null
  line_length: number | null
  max_list: Record<string, number> | null
  max_targets: number | null
  modes: number | null
  monitor: number | null
  name_length: number | null
  namesx: boolean
  message_ref_types: string[] | null
  max_nick_length: number | null
  network: string | null
  nick_length: number | null
  prefix: [string, string][] | null
  rp_channel: string | null
  rp_user: string | null
  remove: boolean
  safe_list: boolean
  safe_rate: boolean
  secure_list: number | null
  silence: number | null
  status_message: string | null
  target_max: [string, number | null][] | null
  topic_length: number | null
  uhnames: boolean
  user_ip: boolean
  user_length: number | null
  user_modes: string[] | null
  utf8_mapping: string | null
  utf8_only: boolean
  vapid: string | null
  vbanlist: boolean
  vlist: string | null
  watch: number | null
  whox: boolean
}

export interface TextMessage {
  content: string
  reactions: Record<string, string[]>
  reply: MessageReference | null
  redacted: boolean
  edited: boolean
  relayed_by: string | null
}

export interface User {
  nickname: string
  username: string | null
  realname: string | null
  display_name: string | null
  description: string | null
  profile_picture_url: string | null
  bot: boolean
}

export type ChannelRole = "Owner" | "Admin" | "Operator" | "HalfOperator" | "Voice" | "Regular"

export type MessageType = "Privmsg" | "Notice" | "Action" | "Join" | "Part" | "Quit"

export type OrbitErrorKind = "NickTaken" | "SaslFailed" | "CapabilityDisabled" | "NotFound" | "Generic" | "UnknownServer" | "Serialize" | "Unknown"

export type ServerEvent =
  | { tag: "Joined"; value: Channel }
  | { tag: "ChannelUpdated"; value: ChannelMetadata }
  | { tag: "ServerInfo"; value: ServerMetadata }
  | { tag: "UserList"; value: { channel: string; users: ChannelUser[] } }
  | { tag: "Message"; value: { channel: string; message: Message } }
  | { tag: "React"; value: { target_message: string; user: string; text: string; is_unreact: boolean } }

export type SignedIn = "User" | "Guest"

export function chat_channel_history_before(server_id: number, channel: string, before_msgid: string): Promise<History>

export function chat_channel_join(server_id: number, channel: string, password?: string | null): Promise<Channel>

export function chat_channel_send_message(server_id: number, channel_name: string, text: string): Promise<Message>

export function init(): void

export function initialize_orbit(): Promise<Server[]>

export function server_channel_list(server_id: number): Promise<ChannelInfo[]>

export function server_connect(url: string): Promise<Server>

export function server_on_data(server_id: number, f: (event: ServerEvent) => void): Promise<void>

export function server_on_disconnect(server_id: number, f: (event: string) => void): Promise<void>

export function server_on_error(server_id: number, f: (error: OrbitError) => void): Promise<void>

export function server_sign_in(server_id: number, nick: string, user: string, realname: string, password: string): Promise<SignedIn>

export function server_sign_in_anonymous(server_id: number, nick: string, user: string, realname: string): Promise<SignedIn>

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module

export interface InitOutput {
  readonly memory: WebAssembly.Memory
  readonly chat_channel_history_before: (a: number, b: number, c: number, d: number, e: number) => any
  readonly chat_channel_join: (a: number, b: number, c: number, d: number, e: number) => any
  readonly chat_channel_send_message: (a: number, b: number, c: number, d: number, e: number) => any
  readonly init: () => void
  readonly initialize_orbit: () => any
  readonly server_channel_list: (a: number) => any
  readonly server_connect: (a: number, b: number) => any
  readonly server_on_data: (a: number, b: any) => any
  readonly server_on_disconnect: (a: number, b: any) => any
  readonly server_on_error: (a: number, b: any) => any
  readonly server_sign_in: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => any
  readonly server_sign_in_anonymous: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => any
  readonly wasm_bindgen__convert__closures_____invoke__h69af921b525aa026: (a: number, b: number, c: any) => [number, number]
  readonly wasm_bindgen__convert__closures_____invoke__hfc4f4eb54d361465: (a: number, b: number, c: any) => [number, number]
  readonly wasm_bindgen__convert__closures_____invoke__h55ccb1222fc95a46: (a: number, b: number, c: any, d: any) => void
  readonly wasm_bindgen__convert__closures_____invoke__h2f5a6b754968c579: (a: number, b: number, c: any) => void
  readonly wasm_bindgen__convert__closures_____invoke__h04b81f5efcb0df6f: (a: number, b: number, c: any) => void
  readonly wasm_bindgen__convert__closures_____invoke__h2f5a6b754968c579_3: (a: number, b: number, c: any) => void
  readonly wasm_bindgen__convert__closures_____invoke__h2f5a6b754968c579_5: (a: number, b: number, c: any) => void
  readonly wasm_bindgen__convert__closures_____invoke__he561c48a3e9220eb: (a: number, b: number) => void
  readonly wasm_bindgen__convert__closures_____invoke__hd7c0f5330375ec49: (a: number, b: number) => void
  readonly wasm_bindgen__convert__closures_____invoke__h84f8f095dc5775a9: (a: number, b: number) => void
  readonly __wbindgen_malloc: (a: number, b: number) => number
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number
  readonly __wbindgen_exn_store: (a: number) => void
  readonly __externref_table_alloc: () => number
  readonly __wbindgen_externrefs: WebAssembly.Table
  readonly __externref_drop_slice: (a: number, b: number) => void
  readonly __wbindgen_free: (a: number, b: number, c: number) => void
  readonly __wbindgen_destroy_closure: (a: number, b: number) => void
  readonly __externref_table_dealloc: (a: number) => void
  readonly __wbindgen_start: () => void
}

export type SyncInitInput = BufferSource | WebAssembly.Module

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init(module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>
