import { createApp, type Component } from "vue"
import { router } from "../router"
import { type Platform, PLATFORM_KEY } from "platform"

/**
 * Creates an Orbit app.
 *
 * @param root Root component
 * @param platform Platform adapter
 * @returns Vue application instance
 */
export function createOrbitApp(root: Component, platform: Platform) {
  const app = createApp(root)
  app.use(router)
  app.provide(PLATFORM_KEY, platform)
  return app
}
