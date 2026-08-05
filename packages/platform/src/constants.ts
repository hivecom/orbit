import type { InjectionKey } from "vue"
import type { Platform } from "./types"

export const PLATFORM_KEY: InjectionKey<Platform> = Symbol("orbit-platform")

export const SETTINGS_KEY = "orbit-settings"
export const SHORTCUTS_KEY = "orbit-shortcuts"

export const IRC_UNKNOWN_CHANNEL = "<unknown>"
