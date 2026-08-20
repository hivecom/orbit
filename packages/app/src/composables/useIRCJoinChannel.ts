import { ref } from "vue"
import { useWindowManager, type WindowChat, type WindowLocation } from "../lib/windows"
import { useIrcStore } from "../stores/irc"

export function useIRCJoinChannel() {
  const loading = ref(false)
  const irc = useIrcStore()
  const { replace, focusedWindow } = useWindowManager()

  async function join(serverId: number, channelId: string, forcedLocation?: WindowLocation) {
    loading.value = true

    try {
      // Check whether the channel is already in joined - if yes, we skip
      // `irc.channelJoin` and just replace instead
      const existing = irc.serverChannels.get(serverId)

      if (!existing?.joined.find((item) => item.data.metadata.name === channelId)) {
        await irc.channelJoin(serverId, channelId)
      }

      const _location = forcedLocation ?? focusedWindow.value?.location ?? "f"
      const _serverId = (focusedWindow.value as WindowChat)?.serverId ?? serverId

      await replace(_location, {
        serverId: _serverId,
        // TODO: this will be dynamic once we move beyond IRC
        type: "chat",
        channelId,
      })
      // }
    } catch (e) {
      console.log("Error joining IRC channel via composable", e)
    } finally {
      loading.value = false
    }
  }

  return {
    loading,
    join,
  }
}
