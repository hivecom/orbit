import dayjs from "dayjs"
import { useConfigStore } from "../stores/config"

/**
 * Formats an IRC timestamp
 */
export function useDateFormatter() {
  const config = useConfigStore()

  /**
   * Formats IRC message timestamp based on the user configuration
   */
  function chatTimestamp(unixTimestamp: number) {
    return dayjs(unixTimestamp).format(config.options.appearance_chat_timestamps_format)
  }

  /**
   * Formats a unix timestamp to the `D dddd YYYY` format. For instance `6 October 2026`
   */
  function simple(unixTimestamp: number) {
    return dayjs(unixTimestamp).format("D MMMM YYYY")
  }

  return {
    chatTimestamp,
    simple,
  }
}
