import { useUrlSearchParams } from "@vueuse/core"
import { computed, ref, unref, watch } from "vue"
import { useIrcStore } from "../stores/irc"
import { IRC_UNKNOWN_CHANNEL } from "./constants"

// TODO: how to tell which window currently has focus? Should it just be a
// method which tries to find focus within the HTML element and then create a
// window object from it which gets returned?

export type WindowLocation = "f" | "l" | "r" | "lt" | "lb" | "rt" | "rb"

type WindowChatURLState = `c:${string}:${string}`
type WindowVoiceURLState = `v:${string}`
type WindowEmptyURLState = `e`

type WindowURLState = WindowChatURLState | WindowVoiceURLState | WindowEmptyURLState

export interface WindowChat {
  type: "chat"
  serverId: number
  channelId: string
}

export interface WindowVoice {
  type: "voice"
  channelId: string
}

export interface WindowEmpty {
  type: "empty"
}

export type Window = WindowChat | WindowVoice | WindowEmpty
export type WindowType = Window["type"]
export type WindowState = Partial<Record<WindowLocation, Window>>
export type WindowAndLocation<T = Window> = T & { location: WindowLocation }

////////////////////////////////////////////////////////////////////////

const WIN_STORAGE_KEY = "o-wm-state"
const WIN_URL_KEY = "w1" // Includes a number for versioning
const WIN_LOCATIONS: WindowLocation[] = ["f", "l", "r", "lt", "lb", "rt", "rb"]

export function getDefaultState(): WindowState {
  const irc = useIrcStore()

  // If we're getting default state, it means there is no previous manager state.
  const firstServer = irc.serverData.values().next().value

  if (firstServer) {
    return {
      f: {
        type: "chat",
        serverId: firstServer.id,
        channelId: IRC_UNKNOWN_CHANNEL,
      },
    }
  }

  return {
    f: {
      type: "empty",
    },
  } as const
}

////////////////////////////////////////////////////////////////////////

// Converts window object into a URL search param value
export function serializeWindow(window: Window): WindowURLState {
  switch (window?.type) {
    case "chat":
      return `c:${window.serverId}:${window.channelId}` satisfies WindowChatURLState

    case "voice":
      return `v:${window.channelId}` satisfies WindowVoiceURLState

    default:
    case "empty":
      return `e` satisfies WindowEmptyURLState
  }
}

// Convers a single window into a state object
export function deserializeWindow(encoded: string): Window | undefined {
  if (!encoded || !encoded.includes(":")) {
    return getDefaultState().f
  }

  const [type, ...params] = encoded.split(":")

  switch (type) {
    case "c":
      if (params.length !== 2) return
      return {
        type: "chat",
        serverId: Number(params[0]),
        channelId: params[1]!,
      }

    case "v":
      if (params.length !== 1) return
      return {
        type: "voice",
        channelId: params[0]!,
      }

    case "e":
      return getDefaultState().f
  }
}

// Converts the entire state into a single URL search param value
export function serializeState(state: WindowState): string {
  if (!state || Object.keys(state).length === 0) {
    return "f:e"
  }

  const entries: string[] = []

  for (const location of WIN_LOCATIONS) {
    const window = state[location]
    if (!window) continue

    entries.push(`${location}:${serializeWindow(window)}`)
  }

  return entries.join(";")
}

// Turns a raw URL search param into the state object
export function deserializeState(url: string): WindowState {
  if (!url) return getDefaultState()

  const windows = url.split(";")
  const state: WindowState = {}

  for (const windowRaw of windows) {
    const separatorIndex = windowRaw.indexOf(":")

    if (separatorIndex === -1) continue

    const location = windowRaw.slice(0, separatorIndex) as WindowLocation
    const windowState = deserializeWindow(windowRaw.slice(separatorIndex + 1))

    if (!windowState) continue

    state[location] = windowState
  }

  if (Object.keys(state).length === 0) {
    return getDefaultState()
  }

  return state
}

export function loadInitialState(): WindowState {
  const urlValue = useUrlSearchParams("history")[WIN_URL_KEY]

  if (urlValue) {
    return deserializeState(urlValue.toString())
  }

  try {
    const raw = localStorage.getItem(WIN_STORAGE_KEY)

    if (raw) {
      const parsed = deserializeState(raw)
      return parsed
    }
  } catch {}
  return getDefaultState()
}

// Main window state stored as JSON object in the URL search params. Where
// location is the key (because it's always unique) and value is the
// Window<Type> object.

// Window manager is a global composable, so all its state must be defined
// outside of it
const params = useUrlSearchParams<{ [WIN_URL_KEY]?: string }>("history", { writeMode: "push" })
const windows = ref<WindowState>({})
const focusedWindow = ref<WindowAndLocation | null>(null)
const isEmpty = computed(() => Object.values(windows.value).filter((item) => item && item.type !== "empty").length === 0)
const initialized = ref(false)

export function useWindowManager() {
  // Keep URL -> State in sync
  watch(
    () => params[WIN_URL_KEY],
    (newState) => {
      if (newState) {
        windows.value = deserializeState(newState)
      }
    },
  )

  // // Keep State -> URL in sync
  watch(
    windows,
    (newState) => {
      const serialized = serializeState(newState)
      params[WIN_URL_KEY] = serialized
      localStorage.setItem(WIN_STORAGE_KEY, serialized)
    },
    { deep: true },
  )

  /**
   * Closes a window at a location. The layout will automatically reflow
   */
  function close(location: WindowLocation) {
    if (!windows.value[location] || location === "f") return

    delete windows.value[location]

    switch (location) {
      case "lt": {
        const current = windows.value.lb
        delete windows.value.lb
        windows.value.l = current
        break
      }

      case "lb": {
        const current = windows.value.lt
        delete windows.value.lt
        windows.value.l = current
        break
      }

      case "rt": {
        const current = windows.value.rb
        delete windows.value.rb
        windows.value.r = current
        break
      }

      case "rb": {
        const current = windows.value.rt
        delete windows.value.rt
        windows.value.r = current
        break
      }

      case "l": {
        if (windows.value.r) {
          windows.value = { f: windows.value.r }
        } else {
          const current = unref(windows.value)
          current.l = windows.value.rt
          current.r = windows.value.rb
          delete current.rt
          delete current.rb
          windows.value = current
        }
        break
      }

      case "r": {
        if (windows.value.l) {
          windows.value = { f: windows.value.l }
        } else {
          const current = unref(windows.value)
          current.l = windows.value.lt
          current.r = windows.value.lb
          delete current.lt
          delete current.lb
          windows.value = current
        }
        break
      }
    }
  }

  /**
   * Swaps two windows
   */
  function swap(from: WindowLocation, to: WindowLocation) {
    const fromRaw = windows.value[from]
    const toRaw = windows.value[to]
    const current = windows.value

    current[from] = toRaw
    current[to] = fromRaw
    windows.value = current
  }

  /**
   * Splits a window into two if possible
   */
  function split(from: WindowLocation, split?: Window) {
    if (!split) return

    switch (from) {
      case "f": {
        const current = unref(windows)
        current.l = split
        current.r = getDefaultState().f
        delete current.f
        windows.value = current
        break
      }

      case "l": {
        const current = unref(windows)
        current.lt = split
        current.lb = getDefaultState().f
        delete current.l
        windows.value = current
        break
      }

      case "r": {
        const current = unref(windows)
        current.rt = split
        current.rb = getDefaultState().f
        delete current.r
        windows.value = current
        break
      }
    }
  }

  /**
   * Inserts a new window into a specific location
   */
  function replace(location: WindowLocation, newState: Window) {
    windows.value[location] = newState
  }

  /**
   * Resets state to just a single empty window
   */
  function reset() {
    // FIXME
    windows.value = getDefaultState()
  }

  /**
   * Called when app initializes, as default state requires pinia state
   */
  function init() {
    if (initialized.value) return
    windows.value = loadInitialState()
    initialized.value = true
  }

  return {
    windows,
    focusedWindow,
    isEmpty,
    close,
    split,
    swap,
    replace,
    reset,
    init,
  }
}
