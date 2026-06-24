import { createWebPlatform } from "./web"
import { createDesktopPlatform } from "./desktop"
import type { Platform } from "./types"
import { PLATFORM_KEY } from "./constants"
import { usePlatform } from "./composables"

export { createWebPlatform, type Platform, PLATFORM_KEY, usePlatform, createDesktopPlatform }
