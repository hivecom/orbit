import init, { chat_channel_history_before, chat_channel_join, chat_channel_send_message, initialize_orbit, server_channel_list, server_connect, server_on_data, server_on_disconnect, server_on_error, server_sign_in, server_sign_in_anonymous } from "core-wasm"
import type { IrcPort } from "../types"

// Browser IRC adapter backed by the core-wasm module
export function createIrcPort(): IrcPort {
  return {
    async initialize() {
      await init()
      return initialize_orbit()
    },
    serverConnect: (url) => server_connect(url),
    serverSignIn: (serverId, nick, user, realname, password) => server_sign_in(serverId, nick, user, realname, password),
    serverSignInAnonymous: (serverId, nick, user, realname) => server_sign_in_anonymous(serverId, nick, user, realname),
    serverChannelList: (serverId) => server_channel_list(serverId),
    serverOnData: (serverId, listener) => server_on_data(serverId, listener),
    serverOnDisconnect: (serverId, listener) => server_on_disconnect(serverId, listener),
    serverOnError: (serverId, listener) => server_on_error(serverId, listener),
    channelJoin: (serverId, channel, password) => chat_channel_join(serverId, channel, password),
    channelHistoryBefore: (serverId, channel, beforeMsgid) => chat_channel_history_before(serverId, channel, beforeMsgid),
    channelSendMessage: (serverId, channel, text) => chat_channel_send_message(serverId, channel, text),
  }
}
