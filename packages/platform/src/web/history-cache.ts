import { createMockPlatform } from "../mock"

// TODO: Implement
export function createIndexedDbCachePort() {
  return createMockPlatform("web").historyCache
}
