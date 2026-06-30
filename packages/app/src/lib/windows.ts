import { useUrlSearchParams } from "@vueuse/core"
import { readonly, ref, unref, watch, watchEffect } from "vue"

type WindowLocation = "f" | "l" | "r" | "lt" | "lb" | "rt" | "rb"

type WindowChatURLState = `c:${string}:${string}`
type WindowVoiceURLState = `v:${string}`
type WindowEmptyURLState = `e`

type WindowURLState = WindowChatURLState | WindowVoiceURLState | WindowEmptyURLState

interface WindowChat {
  type: "chat"
  serverId: string
  channelId: string
}

interface WindowVoice {
  type: "voice"
  channelId: string
}

interface WindowEmpty {
  type: "empty"
}

export type Window = WindowChat | WindowVoice | WindowEmpty
export type WindowType = Window["type"]
export type WindowState = Partial<Record<WindowLocation, Window>>

////////////////////////////////////////////////////////////////////////

const WIN_STORAGE_KEY = "o-wm-state"
const WIN_URL_KEY = "w1" // Includes a number for versioning
const WIN_LOCATIONS: WindowLocation[] = ["f", "l", "r", "lt", "lb", "rt", "rb"]
const WIN_DEFAULT_STATE: WindowState = {
  f: {
    type: "empty",
  },
}

////////////////////////////////////////////////////////////////////////

// Converts window object into a URL search param value
export function serializeWindow(window: Window): WindowURLState {
  switch (window.type) {
    case "chat":
      return `c:${window.serverId}:${window.channelId}` satisfies WindowChatURLState

    case "voice":
      return `v:${window.channelId}` satisfies WindowVoiceURLState

    case "empty":
      return `e` satisfies WindowEmptyURLState
  }
}

// Convers a single window into a state object
export function deserializeWindow(encoded: string): Window | undefined {
  const [_, type, ...params] = encoded.split(":")

  switch (type) {
    case "c":
      if (params.length !== 2) return
      return {
        type: "chat",
        serverId: params[0],
        channelId: params[1],
      }

    case "v":
      if (params.length !== 1) return
      return {
        type: "voice",
        channelId: params[0],
      }

    case "e":
      return {
        type: "empty",
      }
  }
}

// Cibverts the entire state into a single URL search param value
function serializeState(state: WindowState): string {
  const entries: string[] = []

  for (const location of WIN_LOCATIONS) {
    const window = state[location]
    if (!window) continue

    entries.push(`${location}:${serializeWindow(window)}`)
  }

  return entries.join(";")
}

// FIX:
function deserializeState(url: string): WindowState {
  const windows = url.split(";")
  const state: WindowState = {}

  for (const windowRaw of windows) {
    const [location, windowPartition] = windowRaw.split(":") as [WindowLocation, string]
    const windowState = deserializeWindow(windowPartition)

    if (!windowState) continue

    state[location] = windowState
  }

  if (windows.length === 0) {
    return { ...WIN_DEFAULT_STATE }
  }

  return state
}

function loadInitialState(urlValue?: string): WindowState {
  if (urlValue) {
    return deserializeState(urlValue)
  }

  try {
    const raw = localStorage.getItem(WIN_STORAGE_KEY)

    if (raw) {
      return deserializeState(JSON.parse(raw))
    }
  } catch {}
  return { ...WIN_DEFAULT_STATE }
}

// Main window state stored as JSON object in the URL search params. Where
// location is the key (because it's always unique) and value is the
// Window<Type> object.
export function useTiler() {
  const params = useUrlSearchParams<{ [WIN_URL_KEY]?: string }>("history", {
    writeMode: "push",
  })

  const state = ref<WindowState>(loadInitialState(params[WIN_URL_KEY]))

  // Keep URL -> State in sync
  watch(
    () => params[WIN_URL_KEY],
    (newState) => {
      if (newState) {
        state.value = deserializeState(newState)
      }
    },
  )

  // Keep State -> URL in sync
  watch(
    state,
    (newState) => {
      const serialized = serializeState(newState)
      params[WIN_URL_KEY] = serialized
      localStorage.setItem(WIN_STORAGE_KEY, serialized)
    },
    { deep: true },
  )

  // Closes a window
  // function close(location: WindowLocation) {}

  // // Swaps two windows
  // function move(from: WindowLocation, to: WindowLocation) {}

  // Places a new empty pane to a location
  function split(from: WindowLocation, split?: Window) {
    if (!split) return

    switch (from) {
      case "f": {
        const current = unref(state)
        current.l = split
        current.r = { ...WIN_DEFAULT_STATE.f! }
        delete current.f
        state.value = current
        break
      }

      case "l": {
        const current = unref(state)
        current.lt = split
        current.lb = { ...WIN_DEFAULT_STATE.f! }
        delete current.l
        state.value = current
        break
      }

      case "r": {
        const current = unref(state)
        current.rt = split
        current.rb = { ...WIN_DEFAULT_STATE.f! }
        delete current.r
        state.value = current
        break
      }
    }
  }

  watchEffect(() => {
    console.log("STATE UPDATE", state.value)
  })

  return {
    tileset: readonly(state),
    // close,
    // move,
    split,
  }
}
