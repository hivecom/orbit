import { createMockPlatform } from "./mock"
import type { Platform } from "./types"

// TODO: This needs to be implemented immediately when Tauri work begins. For
// now we fallback to the web implementation and changing the target as per
// @zealprince's suggestion
export function createDesktopPlatform(): Platform {
  return {
    ...createMockPlatform("web"),
    target: "desktop",
  }
}
