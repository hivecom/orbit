import { defineStore } from "pinia"
import { shallowRef } from "vue"

/**
 * Orbit configuration
 */
export const useConfigStore = defineStore("config", () => {
  const initialized = shallowRef(false)

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
  }
})
