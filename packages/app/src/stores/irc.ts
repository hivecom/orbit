import { defineStore } from "pinia"
import type { Message, Server, OrbitError, ChannelInfo, Channel } from "core/types"
import { computed, ref, shallowRef } from "vue"
import { usePlatform } from "platform"
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
  const { irc } = usePlatform()
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

    await irc.serverSignInAnonymous(server.id, user.me.displayName, user.me.accountName, user.me.accountName)

    await irc.serverChannelList(server.id).then((channels) => {
      const data = serverChannels.value.get(server.id)
      if (!data) return
      data.available = channels.sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()))
    })

    registerServerEvents(server.id)
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
    const state = await irc.serverConnect(url).catch((e: OrbitError) => {
      throw new Error(e.description)
    })
    await initializeServer(state)
    return state
  }

  function registerServerEvents(serverId: number) {
    // Runs whenever some dataset on the server object changes
    void irc.serverOnData(serverId, (event) => {
      if (event.tag === "Message") {
        const messageKey = `${serverId}:${event.value.channel}`
        const messages = serverMessages.value.get(messageKey)
        if (!messages) {
          serverMessages.value.set(messageKey, [event.value.message])
          return
        }
        messages.push(event.value.message)
        messages.sort((a: Message, b: Message) => a.metadata.server_time - b.metadata.server_time)
      } else if (event.tag === "UserList") {
        const affected = getServerChannel(serverId, event.value.channel)
        if (!affected) return
        affected.users = event.value.users
      } else if (event.tag === "React") {
        console.log("Received reaction", event.value)
      } else {
        console.log("=====")
        console.log("Received unknown dataset", event)
      }
    })

    // Leaving server - clean up state
    void irc.serverOnDisconnect(serverId, (reason) => {
      console.log("Disconnected", reason)
      serverData.value.delete(serverId)
    })

    void irc.serverOnError(serverId, (error) => {
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

      const history = await irc.channelHistoryBefore(serverId, channelId, oldestId.metadata.msgid)
      if (!history) return

      const messages = serverMessages.value.get(messageId)
      if (!messages) return

      messages.push(...history.messages)
      messages.sort((a, b) => a.metadata.server_time - b.metadata.server_time)
    } catch (e: unknown) {
      console.error("Error when requesting scrollback", e as OrbitError)
    }
  }

  /**
   * Joins a channel in an existing server
   */
  async function channelJoin(serverId: number, channelId: string) {
    try {
      const channels = serverChannels.value.get(serverId)
      if (!channels) return
      const data = await irc.channelJoin(serverId, channelId)
      if (!data) return

      // Add channel to joined, remove it from available
      channels.joined.push(data)
      channels.joined.sort((a, b) => a.metadata.name.toLowerCase().localeCompare(b.metadata.name.toLowerCase()))
      channels.available = channels.available.filter((item) => item.name !== data.metadata.name)

      // Upon joining, show backlog
      serverMessages.value.set(`${serverId}:${channelId}`, data.messages)
    } catch (e: any) {
      console.error("Error when joining channel", e as OrbitError)
    }
  }

  /**
   * Leaves a channel and removes all of its stored information
   */
  async function channelLeave(serverId: number, channelId: string) {
    void serverId
    void channelId
    return null
    //   // TODO: Wait for jokler to implement chat_channel_leave() and call it here
    // try {
    //   const channels = serverChannels.value.get(serverId)
    //   if (!channels) return
    //   channels.joined = channels?.joined.filter((c) => c.metadata.name !== channelId)
    //   serverMessages.value.delete(`${serverId}:${channelId}`)

    // } catch (e) {
    //   console.error(e as OrbitError)
    // }
  }

  async function sendMessage(serverId: number, channelId: string, message: string) {
    await irc.channelSendMessage(serverId, channelId, message).catch((e: OrbitError) => {
      throw new Error(e.description)
    })
  }

  return {
    init,
    serverConnect,
    channelJoin,
    channelLeave,
    sendMessage,
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
