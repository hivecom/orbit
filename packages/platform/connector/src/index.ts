import { createWebPlatform } from "./web"
import { createDesktopPlatform } from "./desktop"
import type { Platform } from "./types"
import { PLATFORM_KEY } from "./constants"
import { usePlatform } from "./composables"
import init, { initialize_orbit } from "core-wasm"
import type { Server } from "core-wasm"

export { createWebPlatform, type Platform, type Server, PLATFORM_KEY, usePlatform, createDesktopPlatform }

export async function initOrbit() {
  await init()
  return await initialize_orbit()
}
