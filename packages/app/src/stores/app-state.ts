import { defineStore } from "pinia"
import { useIrcStore } from "./irc"
import { computed, shallowRef } from "vue"
import { useUserStore } from "./user"

/**
 * Tracks global state such as initialization, errors & etc
 */
export const useAppStateStore = defineStore("app-state", () => {
  const ircStore = useIrcStore()
  const userStore = useUserStore()

  const globalError = shallowRef<string | null>(null)
  const ircErrors = shallowRef<string[]>([])

  /**
   * Contains the `initialize` state from all stores, where initialization might
   * block loading of the application with critical resources
   */
  const initialized = computed(() => {
    return ircStore.initialized && userStore.initialized
  })

  return {
    initialized,
    globalError,
    ircErrors,
  }
})
