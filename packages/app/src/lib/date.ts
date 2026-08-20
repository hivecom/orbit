import dayjs from "dayjs"
import { useConfigStore } from "../stores/config"

/**
 * Formats an IRC timestamp
 */
export function formatTimestamp() {}

export function useDateFormatter() {
  const config = useConfigStore()

  /**
   * Formats IRC message timestamp
   */
  function chatTimestamp(unixInSeconds: number) {
    return dayjs(unixInSeconds * 1000).format(config.options.appearance_chat_timestamps_format)
  }

  // function chatDisplay() {}

  return {
    chatTimestamp,
  }
}
