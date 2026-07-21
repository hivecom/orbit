import { createApp, type Component } from "vue"
import { router } from "../router/router"
import { type Platform, PLATFORM_KEY } from "platform"
import { createPinia } from "pinia"
import { setColorTheme } from "@dolanske/vui"
import init, { initialize_orbit } from "core-wasm"
import { useIrcStore } from "../stores/irc"
import { useAppStateStore } from "../stores/app-state"
import { useUserStore } from "../stores/user"

/**
 * Creates the Orbit application and initializes the UI & connectors.
 *
 * @param root Root component
 * @param platform Platform adapter
 * @returns Vue application instance
 */
export async function createOrbitApp(root: Component<any, any, any, any, any>, platform: Platform) {
  const app = createApp(root)
  const pinia = createPinia()

  setColorTheme("dark")

  app.use(router)
  app.use(pinia)
  app.provide(PLATFORM_KEY, platform)

  // 1. run connector inititilization code
  // 2. pass returned data to the related stores and run their `init` functions.
  //    Each data holding store contains an init function which takes in the
  //    initial dataset and populates the state. After that, it registers a data
  //    update listener which will subsqeuently update all the state on change

  //    2.1 Handle server capabilities
  //    2.2 Handle other server & channel state

  // non-blocking operation, app receives a loading spinner while this is happening
  await init().then(async () => {
    return initialize_orbit()
      .then(async (controller) => {
        const userStore = useUserStore()
        userStore.init()

        const ircStore = useIrcStore(pinia)
        await ircStore.init(controller)
      })
      .catch((e) => {
        const appState = useAppStateStore()
        console.log("Failed to initialize orbit", e)
        appState.globalError = "Failed to initialize Orbit. Check console for errors."
      })
  })

  console.log("WASM startup completed")

  return app
}
