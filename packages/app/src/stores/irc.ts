import { defineStore } from "pinia"
import type { IrcServer, ServerList } from "../../../core/core-wasm/pkg/core_wasm"
import { shallowRef } from "vue"

/**
 * Global store handling all IRC data and hands it to the UI for consumption.
 */
export const useIrcStore = defineStore("irc", () => {
  const initialized = shallowRef(false)
  const servers = shallowRef<IrcServer[]>([])

  let controller: ServerList = {} as ServerList

  /**
   * Every global store ships with an init function which is always called in
   * the `createOrbitApp` and nowhere else. Takes in the initial dataset
   * returned by the IRC/Depot/etc servers.
   */
  async function init(_controller: ServerList) {
    controller = _controller

    const _servers = await controller.get_servers()
    servers.value = _servers
    // TODO: handle capabilities
  }

  return {
    init,
    initialized,
    servers,
    controller,
  }
})
