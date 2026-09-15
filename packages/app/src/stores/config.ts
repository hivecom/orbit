import { onKeyStroke, useLocalStorage, useMagicKeys, whenever } from "@vueuse/core"
import { defineStore } from "pinia"
import { effectScope, onBeforeUnmount, reactive, shallowRef } from "vue"
import type { KeyboardShortcuts, ShortcutCallback } from "../types/config"
import { SETTINGS_KEY } from "../lib/constants"

const config = {
  appearance_global_zen_enabled: false,
  appearance_chat_colored_usernames: false,
  appearance_chat_timestamps_enabled: true,
  appearance_chat_timestamps_format: "HH:mm:ss",
  appearance_chat_center_chat: true,
  appearance_chat_width: 100,
}

// NOTE: Keymap currently cannot be changed. We'll implement it once we have
// persistent settings for users
const keymap = {
  "global:navigation-toggle": {
    keys: "Ctrl+Shift+S",
    handler: (e) => e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "s",
    title: "Navigation toggle",
    description: "Controls the sidebar open/closed state",
  },
} satisfies KeyboardShortcuts

type Shortcut = keyof typeof keymap

/**
 * Orbit configuration
 */
export const useConfigStore = defineStore("config", () => {
  const initialized = shallowRef(false)
  const options = useLocalStorage(SETTINGS_KEY, config, { mergeDefaults: true })

  /**
   * Every global store ships with an init function which is always called in
   * the `createOrbitApp` and nowhere else. Takes in the initial dataset
   * returned by the IRC/Depot/etc servers.
   */
  function init() {
    initialized.value = true

    // Register watcher scope for application-wide shortcut handling. We need to use a scope
    // because this registration happens outside of a specific vue component
    const watcherScope = effectScope()

    watcherScope.run(() => {
      const keys = useMagicKeys()

      for (const [id, shortcut] of Object.entries(keymap)) {
        if (shortcut.handler) {
          onKeyStroke(shortcut.handler, (e) => e.preventDefault())
        }

        whenever(keys[shortcut.keys], () => {
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
