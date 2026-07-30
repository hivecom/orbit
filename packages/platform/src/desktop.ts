import { createMockPlatform } from "./mock"
import type { Platform } from "./types"

// TODO: implement
export function createDesktopPlatform(): Platform {
  return createMockPlatform("desktop")
}
