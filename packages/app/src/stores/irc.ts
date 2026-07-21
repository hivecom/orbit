import { defineStore } from "pinia"
import { Message, React, type IrcConnection, type Server, type ServerList } from "core-wasm"
import { computed, ref, shallowRef } from "vue"
import { useUserStore } from "./user"
import { useAppStateStore } from "./app-state"

export const IRC_UNKNOWN = "<unknown>"

/**
 * Global store handling all IRC data and hands it to the UI for consumption.
 */
export const useIrcStore = defineStore("irc", () => {
  const user = useUserStore()
  const app = useAppStateStore()

  const initialized = shallowRef(false)

  // Needs to be a ref, as deep properties will be dynamically updated
  const serverState = ref<Map<number, Server>>(new Map())

  // Is shallow, as it's only set once on connection or disconnect
  const serverHandlers = shallowRef<Map<number, IrcConnection>>(new Map())

  // Holds references to messages per server. This should be actually per `server:channel`
  const serverMessages = shallowRef<Map<number, Message[]>>(new Map())

  let controller: ServerList = {} as ServerList

  /**
   * Every global store ships with an init function which is always called in
   * the `createOrbitApp` and nowhere else. Takes in the initial dataset
   * returned by the IRC/Depot/etc servers.
   */
  async function init(_controller: ServerList) {
    controller = _controller

    console.log("Initial orbit servers", controller.servers.length)

    // Get server state and save their data & controllers
    await Promise.allSettled(
      controller.servers.map((serv) => {
        return serv.state()
      }),
    ).then((results) => {
      for (let i = 0; i < results.length; i++) {
        const result = results[i]
        if (result.status === "fulfilled") {
          const key = result.value.id
          serverState.value.set(key, result.value)
          serverHandlers.value.set(key, controller.servers[i])
        }
      }
    })

    initialized.value = true
  }

  /**
   * Connects to the server address
   */
  async function serverConnect(url: string) {
    const handler = await controller.connect(url).catch((e) => {
      throw new Error(e)
    })

    const state = await handler.state()
    console.log("Received server state", state.toJSON())
    serverState.value.set(state.id, state)
    serverHandlers.value.set(state.id, handler)

    await handler.sign_in_anonymous(user.me.displayName, user.me.accountName, user.me.accountName)
    console.log("Signed in")

    registerServerEvents(state.id, handler)

    return {
      handler,
      state,
    }
  }

  function registerServerEvents(key: number, handler: IrcConnection) {
    // Runs whenever some dataset on the server object changes
    handler.on_data((event) => {
      if (event instanceof Message) {
        const existing = serverMessages.value.get(key) ?? []
        existing.push(event)
        serverMessages.value.set(key, existing)
      } else if (event instanceof React) {
        // TODO
        console.log("Received reaction", event)
      }
    })

    // Laving server - clean up state
    handler.on_disconnect((reason) => {
      console.log("Disconnected", reason)
      serverHandlers.value.delete(key)
      serverState.value.delete(key)
    })

    handler.on_error((error) => {
      app.ircErrors.push(error)
    })
  }

  function getServerState(id: number) {
    return computed(() => serverState.value.get(id))
  }

  return {
    init,
    serverConnect,
    initialized,
    controller,
    serverData: serverState,
    serverControllers: serverHandlers,
    getServerState,
  }
})
