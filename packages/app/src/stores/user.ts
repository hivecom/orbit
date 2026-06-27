import { defineStore } from "pinia"

/**
 * User related data
 */
export const useIrcStore = defineStore("user", () => {
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
