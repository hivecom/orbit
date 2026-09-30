import { ref } from "vue"
import { useWindowManager, type Window, type WindowChat, type WindowLocation } from "../lib/windows"
import { useIrcStore } from "../stores/irc"

interface JoinOptions {
  forceLocation?: WindowLocation
  split?: boolean
}

export function useIRCJoinChannel() {
  const loading = ref(false)
  const irc = useIrcStore()
  const { replace, focusedWindow, split } = useWindowManager()

  async function join(serverId: number, channelId: string, options: JoinOptions = {}) {
    loading.value = true

    try {
      // Check whether the channel is already in joined - if yes, we skip
      // `irc.channelJoin` and just replace instead
      const existing = irc.serverChannels.get(serverId)

      if (!existing?.joined.find((item) => item.data.metadata.name === channelId)) {
        await irc.channelJoin(serverId, channelId)
      }

      const _location = options.forceLocation ?? focusedWindow.value?.location ?? "f"
      const _serverId = serverId ?? (focusedWindow.value as WindowChat)?.serverId

      const payload: Window = {
        serverId: _serverId,
        // NOTE: this will be dynamic once we move beyond IRC
        type: "chat",
        channelId,
      }

      if (options.split && focusedWindow.value) {
        split(_location, payload)
      } else {
        await replace(_location, payload)
      }
    } catch (e) {
      console.error("Error joining IRC channel via composable", e)
    } finally {
      loading.value = false
    }
  }

  return {
    loading,
    join,
  }
}
