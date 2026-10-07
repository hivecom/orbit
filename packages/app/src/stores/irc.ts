import { defineStore } from "pinia"
import { type Message, type Server, type OrbitError, type ChannelInfo, type Channel, server_sign_in_anonymous, server_channel_list, server_connect, server_on_data, server_on_disconnect, server_on_error, chat_channel_history_before, chat_channel_join } from "core-wasm"
import { computed, ref, shallowRef } from "vue"
import { useUserStore } from "./user"
import { useAppStateStore } from "./app-state"
import { searchString } from "@dolanske/vui"

export interface IrcChannels {
  joined: Channel[]
  available: ChannelInfo[]
}

export type ServerWithGroupedChannels = Server & {
  groupedChannels: IrcChannels
}

/**
 * Global store handling all IRC data and hands it to the UI for consumption.
 */
export const useIrcStore = defineStore("irc", () => {
  const user = useUserStore()
  const app = useAppStateStore()
  const initialized = shallowRef(false)

  // Holds reference to server metadata
  const serverData = ref<Map<number, Server>>(new Map())
  const serverChannels = ref<Map<number, IrcChannels>>(new Map())
  const serverMessages = ref<Map<string, Message[]>>(new Map())

  /**
   * Returns the servers including their group (joined/available) channels.
   * Because the state is in a class, we have to convert it to a plain object.
   * So none of the methods on the server & channel objects should be used.
   */
  const serversWithChannels = computed(() => {
    return [...serverData.value.values()].map((server) => {
      return {
        ...server,
        groupedChannels: serverChannels.value.get(server.id),
      }
    }) as ServerWithGroupedChannels[]
  })

  function filterServersWithChannels(search: string) {
    if (!search) return serversWithChannels.value

    return serversWithChannels.value.map((server) => {
      return {
        // oxlint-disable-next-line typescript/no-misused-spread
        ...server,
        groupedChannels: {
          joined: server.groupedChannels.joined.filter((channel) => searchString(channel.metadata.name, search)),
          available: server.groupedChannels.available.filter((channel) => searchString(channel.name, search)),
        },
      }
    }) as ServerWithGroupedChannels[]
  }

  /**
   * Initializes empty server datasets and fetches available (unjoined channels)
   */
  async function initializeServer(server: Server) {
    serverData.value.set(server.id, server)
    serverChannels.value.set(server.id, { joined: [], available: [] })

    await server_sign_in_anonymous(server.id, user.me.displayName, user.me.accountName, user.me.accountName)

    await server_channel_list(server.id).then((channels) => {
      const data = serverChannels.value.get(server.id)
      if (!data) return
      data.available = channels.sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()))
      serverChannels.value.set(server.id, data)
    })
  }

  /**
   * Every global store ships with an init function which is always called in
   * the `createOrbitApp` and nowhere else. Takes in the initial dataset
   * returned by the IRC/Depot/etc servers.
   */
  async function init(servers: Server[]) {
    // Initialize servers
    await Promise.allSettled(servers.map(initializeServer))
    initialized.value = true
  }

  /**
   * Connects to the server address
   *
   * By default it does not connect to any channels. Instead it fetches all
   * unjoined channels and users get to choose the first one they join in the UI.
   */
  async function serverConnect(url: string) {
    const state = await server_connect(url).catch((e) => {
      throw new Error(e)
    })

    await initializeServer(state)
    registerServerEvents(state.id)

    return state
  }

  function registerServerEvents(serverId: number) {
    // Runs whenever some dataset on the server object changes
    void server_on_data(serverId, (event) => {
      if (event.tag === "Privmsg") {
        const messageKey = `${serverId}:${event.value.channel}`
        const messages = serverMessages.value.get(messageKey) ?? []
        messages.push(event.value.message)
        messages.sort((a: Message, b: Message) => a.metadata.server_time - b.metadata.server_time)
        serverMessages.value.set(messageKey, messages)
      } else if (event.tag === "React") {
        console.log("Received reaction", event.value)
      } else {
        console.log("=====")
        console.log("Received unknown dataset", event)
      }
    })

    // Leaving server - clean up state
    void server_on_disconnect(serverId, (reason) => {
      console.log("Disconnected", reason)
      serverData.value.delete(serverId)
    })

    void server_on_error(serverId, (error) => {
      app.ircErrors.push(error)
    })
  }

  function getServerState(serverId: number) {
    return serverData.value.get(serverId)
  }

  function getChannelMessages(serverId: number, channelId: string) {
    const messageKey = `${serverId}:${channelId}`
    return serverMessages.value.get(messageKey)
  }

  function getServerChannels(serverId: number) {
    return serverChannels.value.get(serverId)
  }

  function getServerChannel(serverId: number, channelId: string) {
    return serverChannels.value.get(serverId)?.joined.find((channel) => channel.metadata.name === channelId)
  }

  async function requestScrollback(serverId: number, channelId: string) {
    const messageId = `${serverId}:${channelId}`

    try {
      const oldestId = serverMessages.value.get(messageId)?.[0]
      if (!oldestId) return

      const history = await chat_channel_history_before(serverId, channelId, oldestId.metadata.msgid)
      if (!history) return

      const messages = serverMessages.value.get(messageId)
      if (!messages) return

      messages.push(...history.messages)
      messages.sort((a, b) => a.metadata.server_time - b.metadata.server_time)
      serverMessages.value.set(messageId, messages)
    } catch (e: unknown) {
      console.error(e as OrbitError)
    }
  }

  /**
   * Joins a channel in an existing server
   */
  async function channelJoin(serverId: number, channelId: string) {
    try {
      const channels = serverChannels.value.get(serverId)
      if (!channels) return
      const data = await chat_channel_join(serverId, channelId)
      if (!data) return

      console.log(data)

      // Add channel to joined, remove it from available
      channels.joined.push(data)
      channels.joined = [...channels.joined].sort((a, b) => a.metadata.name.toLowerCase().localeCompare(b.metadata.name.toLowerCase()))
      channels.available = channels.available.filter((item) => item.name !== data.metadata.name)

      // Upon joining, show backlog
      serverMessages.value.set(`${serverId}:${channelId}`, data.messages)

      serverChannels.value.set(serverId, channels)
    } catch (e: any) {
      console.error(e as OrbitError)
    }
  }

  return {
    init,
    serverConnect,
    channelJoin,
    initialized,
    serverData,
    getServerState,
    getChannelMessages,
    getServerChannels,
    getServerChannel,
    requestScrollback,
    serverChannels,

    // Servers containing channels list & filtering
    serversWithChannels,
    filterServersWithChannels,
  }
})
