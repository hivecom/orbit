import { inject } from "vue"
import type { Platform } from "./types"
import { PLATFORM_KEY } from "./constants"

/**
 * Access the injected platform adapter from a component or composable.
 * */
export function usePlatform(): Platform {
  const platform = inject<Platform>(PLATFORM_KEY)
  if (!platform) {
    throw new Error("No platform adapter provided. Make sure to provide a platform adapter during Orbit app initialization.")
  }
  return platform
}
