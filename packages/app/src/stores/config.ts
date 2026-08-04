import { useLocalStorage } from "@vueuse/core"
import { defineStore } from "pinia"
import { SETTINGS_KEY } from "platform/src/constants"
import { shallowRef } from "vue"

const getDefaultConfig = () => ({
  appearance_global_zen: false,
  appearance_chat_colored_usernames: false,
  appearance_chat_timestamps_enabled: true,
  appearance_chat_timestamps_format: "hh:mm:ss",
})

/**
 * Orbit configuration
 */
export const useConfigStore = defineStore("config", () => {
  const initialized = shallowRef(false)
  const options = useLocalStorage(SETTINGS_KEY, () => getDefaultConfig())

  /**
   * Every global store ships with an init function which is always called in
   * the `createOrbitApp` and nowhere else. Takes in the initial dataset
   * returned by the IRC/Depot/etc servers.
   */
  function init() {
    initialized.value = true
  }

  return {
    init,
    initialized,
    options,
  }
})
