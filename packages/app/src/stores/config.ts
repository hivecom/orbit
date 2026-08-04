import { useLocalStorage, useMagicKeys, whenever } from "@vueuse/core"
import { defineStore } from "pinia"
import { SHORTCUTS_KEY } from "platform/src/constants"
import { effectScope, onBeforeUnmount, reactive, shallowRef } from "vue"

const getDefaultConfig = () => ({
  appearance_global_zen_enabled: false,
  appearance_chat_colored_usernames: false,
  appearance_chat_timestamps_enabled: true,
  appearance_chat_timestamps_format: "hh:mm:ss",
  appearance_chat_center_chat: true,
  appearance_chat_width: 100,
})

type Shortcut = "global:navigation-toggle"
type ShortcutCallback = () => void

const getDefaultShortcuts = (): Record<Shortcut, string> => ({
  "global:navigation-toggle": "Ctrl+Shift+S",
})

export const shortcutMeta: Record<Shortcut, { title: string; description: string }> = {
  "global:navigation-toggle": {
    title: "Navigation toggle",
    description: "Controls the sidebar open/closed state",
  },
}

/**
 * Orbit configuration
 */
export const useConfigStore = defineStore("config", () => {
  const initialized = shallowRef(false)
  // TODO: once all options are finalized, move to using localStorage
  // const options = useLocalStorage(SETTINGS_KEY, () => getDefaultConfig())
  const options = reactive(getDefaultConfig())
  const keymap = useLocalStorage(SHORTCUTS_KEY, () => getDefaultShortcuts())

  /**
   * Every global store ships with an init function which is always called in
   * the `createOrbitApp` and nowhere else. Takes in the initial dataset
   * returned by the IRC/Depot/etc servers.
   */
  function init() {
    initialized.value = true

    // Register watcher scope for application-wide shortcut handling
    const watcherScope = effectScope()

    watcherScope.run(() => {
      const keys = useMagicKeys({
        passive: false,
        onEventFired(e) {
          // REVIEW: Since Orbit is an app, we pretty much just prevent all default shortcuts
          e.preventDefault()
          // If a default browser event needs to be prevented,
          // add it into this condition
          //   if (e.ctrlKey && e.key === 'u')
          //     e.preventDefault()
        },
      })

      for (const [id, shortcut] of Object.entries(keymap.value)) {
        whenever(keys[shortcut], () => {
          registeredShortcuts[id].map((fn) => fn())
        })
      }
    })
  }

  const registeredShortcuts = reactive<Record<string, ShortcutCallback[]>>({})

  function onShortcut(shortcut: Shortcut, callback: ShortcutCallback) {
    if (!registeredShortcuts[shortcut]) {
      registeredShortcuts[shortcut] = [callback]
    } else {
      registeredShortcuts[shortcut].push(callback)
    }

    // Shortcuts should only trigger in active scopes. If the current scope
    // closes, remove all registered shortcuts from this scope per key
    onBeforeUnmount(() => {
      const filtered = registeredShortcuts[shortcut].filter((cb) => cb !== callback)
      registeredShortcuts[shortcut] = filtered
    })
  }

  return {
    init,
    initialized,
    options,
    onShortcut,
    keymap,
  }
})
