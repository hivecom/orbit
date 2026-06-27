import { defineStore } from "pinia"

/**
 * Global store handling all IRC data and hands it to the UI for consumption.
 */
export const useIrcStore = defineStore("irc", () => {
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
