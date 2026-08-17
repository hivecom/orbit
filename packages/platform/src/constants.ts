import type { InjectionKey } from "vue"
import type { Platform } from "./types"

export const PLATFORM_KEY: InjectionKey<Platform> = Symbol("orbit-platform")
