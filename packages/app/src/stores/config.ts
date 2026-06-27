import { defineStore } from "pinia"

/**
 * Orbit configuration
 */
export const useIrcStore = defineStore("config", () => {
  /**
   * Every global store ships with an init function which is always called in
   * the `createOrbitApp` and nowhere else. Takes in the initial dataset
   * returned by the IRC/Depot/etc servers.
   */
  function init() {}

  return {
    init,
  }
})
