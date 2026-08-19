import { ref } from "vue"
import { useWindowManager, type WindowLocation } from "../lib/windows"
import { useIrcStore } from "../stores/irc"

export function useIRCJoinChannel() {
  const loading = ref(false)
  const irc = useIrcStore()
  const { replace, focusedWindow } = useWindowManager()

  async function join(serverId: number, channelId: string, forcedLocation?: WindowLocation) {
    loading.value = true
    console.log(serverId, channelId, focusedWindow.value)
    try {
      if (focusedWindow.value && focusedWindow.value.type === "chat") {
        await irc.channelJoin(serverId, channelId)

        replace(forcedLocation ?? focusedWindow.value.location, {
          serverId: focusedWindow.value.serverId,
          type: focusedWindow.value.type,
          channelId,
        })
      }
    } catch (e) {
      console.log(e)
    } finally {
      loading.value = false
    }
  }

  return {
    loading,
    join,
  }
}
