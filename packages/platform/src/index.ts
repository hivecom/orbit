import { createWebPlatform } from "./web"
import { createDesktopPlatform } from "./desktop"
import type { IrcPort, Platform } from "./types"
import { PLATFORM_KEY } from "./constants"
import { usePlatform } from "./composables"

export { createWebPlatform, type Platform, type IrcPort, PLATFORM_KEY, usePlatform, createDesktopPlatform }
