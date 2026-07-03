import { defineStore } from "pinia"
import { useIrcStore } from "./irc"
import { computed, shallowRef } from "vue"

/**
 * Tracks global state such as initialization, errors & etc
 */
export const useAppStateStore = defineStore("app-state", () => {
  const ircStore = useIrcStore()
  const globalError = shallowRef<string | null>(null)

  /**
   * Contains the `initialize` state from all stores, where initialization might
   * block loading of the application with critical resources
   */
  const initialized = computed(() => {
    return ircStore.initialized
  })

  return {
    initialized,
    globalError,
  }
})
