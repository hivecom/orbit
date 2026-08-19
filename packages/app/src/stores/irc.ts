import { defineStore } from "pinia"
import { ChannelMessage, Message, React, type IrcConnection, type Server, type ServerList, OrbitError, IrcChannel, ChannelInfo, Channel } from "core-wasm"
import { ref, shallowRef } from "vue"
import { useUserStore } from "./user"
import { useAppStateStore } from "./app-state"

interface IrcChannelWithHandler {
  handler: IrcChannel
  data: Channel
}

/**
 * Global store handling all IRC data and hands it to the UI for consumption.
 */
export const useIrcStore = defineStore("irc", () => {
  const user = useUserStore()
  const app = useAppStateStore()

  const initialized = shallowRef(false)

  // Holds reference to server metadata
  const serverState = ref<Map<number, Server>>(new Map())
  const serverHandlers = ref<Map<number, IrcConnection>>(new Map())

  // Holds channel information per server
  const serverChannels = ref<Map<number, { joined: IrcChannelWithHandler[]; available: ChannelInfo[] }>>(new Map())

  // Holds references to messages per server where the id is `serverId:channelId`
  const serverMessages = ref<Map<string, Message[]>>(new Map())

  let controller: ServerList = {} as ServerList

  /**
   * Initializes empty server datasets and fetches available (unjoined channels)
   */
  async function initializeServer(server: Server, handler: IrcConnection) {
    serverState.value.set(server.id, server)
    serverHandlers.value.set(server.id, handler)
    serverChannels.value.set(server.id, { joined: [], available: [] })

    await handler.sign_in_anonymous(user.me.displayName, user.me.accountName, user.me.accountName)

    await handler.channel_list().then((channels) => {
      const data = serverChannels.value.get(server.id)
      if (!data) return
      data.available = channels
      serverChannels.value.set(server.id, data)
    })
  }

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
      controller.servers.map(async (serv, index) => {
        const server = await serv.state()
        await initializeServer(server, controller.servers[index]!)
        return serv.state()
      }),
    )

    initialized.value = true
  }

  /**
   * Connects to the server address
   *
   * By default it does not connect to any channels. Instead it fetches all
   * unjoined channels and users get to choose the first one they join in the UI.
   */
  async function serverConnect(url: string) {
    const handler = await controller.connect(url).catch((e) => {
      throw new Error(e)
    })

    const state = await handler.state()
    await initializeServer(state, handler)

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
        const messageKey = `${key}:${event.channel}`
        const messages = serverMessages.value.get(messageKey) ?? []
        messages.push(event.message)
        messages.sort((a: Message, b: Message) => a.metadata.server_time - b.metadata.server_time)
        serverMessages.value.set(messageKey, messages)
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

  function getServerState(serverId: number) {
    return serverState.value.get(serverId)
  }

  function getChannelMessages(serverId: number, channelId: string) {
    const messageKey = `${serverId}:${channelId}`
    return serverMessages.value.get(messageKey)
  }

  function getServerChannels(serverId: number) {
    return serverChannels.value.get(serverId)
  }

  function getServerChannel(serverId: number, channelId: string) {
    return serverChannels.value.get(serverId)?.joined.find((channel) => channel.data.metadata.name === channelId)
  }

  async function requestScrollback(serverId: number, channelId: string) {
    const messageId = `${serverId}:${channelId}`

    try {
      const oldestId = serverMessages.value.get(messageId)?.[0].metadata.msgid
      if (!oldestId) return

      const history = await serverHandlers.value.get(serverId)?.history_before(channelId, oldestId)
      if (!history) return

      const messages = serverMessages.value.get(messageId)
      if (!messages) return

      messages.push(...history.messages)
      messages.sort((a, b) => a.metadata.server_time - b.metadata.server_time)
      serverMessages.value.set(messageId, messages)
    } catch (e: unknown) {
      const error = e as OrbitError
      console.error(JSON.parse(error.toString()))
    }
  }

  /**
   * Joins a channel in an existing server
   */
  async function channelJoin(serverId: number, channelId: string) {
    try {
      const serverHandler = serverHandlers.value.get(serverId)
      const channels = serverChannels.value.get(serverId)
      if (!serverHandler || !channels) return
      const handler = await serverHandler.join_channel(channelId)
      const data = (await handler.state())!

      // Add channel to joined, remove it from available
      channels.joined.push({ data, handler })
      channels.available = channels.available.filter((item) => item.name !== data.metadata.name)

      // Upon joining, show backlog
      serverMessages.value.set(`${serverId}:${channelId}`, data.messages)

      serverChannels.value.set(serverId, channels)
    } catch (e: unknown) {
      const error = e as OrbitError
      console.error(JSON.parse(error.toString()))
    }

    // return { data, handler }
  }

  return {
    init,
    serverConnect,
    channelJoin,
    initialized,
    controller,
    serverData: serverState,
    serverControllers: serverHandlers,
    getServerState,
    getChannelMessages,
    getServerChannels,
    getServerChannel,
    requestScrollback,
    serverChannels,
  }
})
