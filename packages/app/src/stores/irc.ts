import { defineStore } from "pinia"
import { ChannelMessage, Message, React, type IrcConnection, type Server, type ServerList, OrbitError, IrcChannel } from "core-wasm"
import { computed, reactive, ref, shallowRef } from "vue"
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
  const serverMessages = reactive<Map<number, Map<string, Message[]>>>(new Map())
  const serverChannel = ref<IrcChannel>()

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
        if (result && result.status === "fulfilled") {
          const key = result.value.id
          serverState.value.set(key, result.value)
          serverHandlers.value.set(key, controller.servers[i]!)
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
    serverChannel.value = await handler.join_channel("#orbit/testing")
    console.log("Signed in")

    // Set initial channel messages
    const channelState = (await serverChannel.value.state())!

    const existingServer = serverMessages.get(state.id) ?? new Map<string, Message[]>()
    const existingChannel = existingServer.get(channelState.metadata.name) ?? []

    existingChannel.push(...channelState.messages)
    existingChannel.sort((a, b) => a.metadata.server_time - b.metadata.server_time)

    existingServer.set(channelState.metadata.name, existingChannel)
    serverMessages.set(state.id, existingServer)

    registerServerEvents(state.id, handler)

    return {
      handler,
      state,
    }
  }

  function registerServerEvents(key: number, handler: IrcConnection) {
    // Runs whenever some dataset on the server object changes
    handler.on_data((event) => {
      if (event instanceof ChannelMessage) {
        const existingServer = serverMessages.get(key) ?? new Map()
        const existingChannel = existingServer.get(event.channel) ?? []

        existingChannel.push(event.message)
        existingChannel.sort((a: Message, b: Message) => a.metadata.server_time - b.metadata.server_time)

        existingServer.set(event.channel, existingChannel)
        serverMessages.set(key, existingServer)
      } else if (event instanceof React) {
        // TODO
        console.log("Received reaction", event)
      }
    })

    // Leaving server - clean up state
    handler.on_disconnect((reason) => {
      console.log("Disconnected", reason)
      serverHandlers.value.delete(key)
      serverState.value.delete(key)
    })

    handler.on_error((error) => {
      app.ircErrors.push(error)
    })
  }

  // TODO: these should be cached not to create a separate computed value on each call
  function getServerState(id: number) {
    return computed(() => serverState.value.get(id))
  }

  function getChannelMessages(id: number, channel: string) {
    return computed(() => serverMessages.get(id)?.get(channel))
  }

  // TODO: will be called automatically by a scroll listener to append new messages as user's nearing the top of the window
  async function requestScrollback(id: number, channel: string) {
    try {
      const oldestId = serverMessages.get(id)?.get(channel)?.at(0)?.metadata.msgid
      if (!oldestId) {
        return
      }
      const history = await serverHandlers.value.get(id)?.history_before(channel, oldestId)

      if (!history) {
        return
      }

      const existingServer = serverMessages.get(id) ?? new Map<string, Message[]>()
      const existingChannel = existingServer.get(history.channel) ?? []

      existingChannel.push(...history.messages)
      existingChannel.sort((a, b) => a.metadata.server_time - b.metadata.server_time)

      existingServer.set(history.channel, existingChannel)
      serverMessages.set(id, existingServer)
    } catch (e: unknown) {
      const error = e as OrbitError
      console.error(JSON.parse(error.toString()))
    }
  }

  return {
    init,
    serverConnect,
    initialized,
    controller,
    serverData: serverState,
    serverControllers: serverHandlers,
    getServerState,
    getChannelMessages,
    requestScrollback,
    serverChannel,
  }
})
