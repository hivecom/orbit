import { createApp, type Component } from "vue"
import { router } from "../router"
import { type Platform, PLATFORM_KEY } from "platform"
import { createPinia } from "pinia"

/**
 * Creates the Orbit application and initializes the UI & connectors.
 *
 * @param root Root component
 * @param platform Platform adapter
 * @returns Vue application instance
 */
export function createOrbitApp(root: Component<any, any, any, any, any>, platform: Platform) {
  const app = createApp(root)

  app.use(router)
  app.use(createPinia())

  app.provide(PLATFORM_KEY, platform)

  // TODO
  // 1. run connector inititilization code
  // 2. pass returned data to the related stores and run their `init` functions.
  //    Each data holding store contains an init function which takes in the
  //    initial dataset and populates the state. After that, it registers a data
  //    update listener which will subsqeuently update all the state on change

  //    2.1 Handle server capabilities
  //    2.2 Handle other server & channel state

  return app
}
