import type { IrcPort, Platform } from "./types"
import { PLATFORM_KEY } from "./constants"
import { usePlatform } from "./composables"

// Shared entry: types and injection helpers only. Targets have their own
// exports eg.: `platform/web` and `platform/desktop`
export { type Platform, type IrcPort, PLATFORM_KEY, usePlatform }
