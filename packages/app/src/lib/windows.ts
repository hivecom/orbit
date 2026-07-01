import { useUrlSearchParams } from "@vueuse/core"
import { readonly, ref, unref, watch } from "vue"

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
  const [type, ...params] = encoded.split(":")

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

// Converts the entire state into a single URL search param value
function serializeState(state: WindowState): string {
  const entries: string[] = []

  for (const location of WIN_LOCATIONS) {
    const window = state[location]
    if (!window) continue

    entries.push(`${location}:${serializeWindow(window)}`)
  }

  return entries.join(";")
}

// Turns a raw URL search param into the state object
function deserializeState(url: string): WindowState {
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
      const parsed = deserializeState(raw)
      return parsed
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

  // // Keep State -> URL in sync
  watch(
    state,
    (newState) => {
      const serialized = serializeState(newState)
      params[WIN_URL_KEY] = serialized
      localStorage.setItem(WIN_STORAGE_KEY, serialized)
    },
    { deep: true, immediate: true },
  )

  /**
   * Closes a window at a location. The layout will automatically reflow
   */
  function close(location: WindowLocation) {
    if (!state.value[location] || location === "f") return

    delete state.value[location]

    switch (location) {
      case "lt": {
        const current = state.value.lb
        delete state.value.lb
        state.value.l = current
        break
      }

      case "lb": {
        const current = state.value.lt
        delete state.value.lt
        state.value.l = current
        break
      }

      case "rt": {
        const current = state.value.rb
        delete state.value.rb
        state.value.r = current
        break
      }

      case "rb": {
        const current = state.value.rt
        delete state.value.rt
        state.value.r = current
        break
      }

      case "l": {
        if (state.value.r) {
          state.value = { f: state.value.r }
        } else {
          const current = unref(state.value)
          current.l = state.value.rt
          current.r = state.value.rb
          delete current.rt
          delete current.rb
          state.value = current
        }
        break
      }

      case "r": {
        if (state.value.l) {
          state.value = { f: state.value.l }
        } else {
          const current = unref(state.value)
          current.l = state.value.lt
          current.r = state.value.lb
          delete current.lt
          delete current.lb
          state.value = current
        }
        break
      }
    }
  }

  /**
   * Swaps two windows
   */
  function swap(from: WindowLocation, to: WindowLocation) {
    const fromRaw = state.value[from]
    const toRaw = state.value[to]
    const current = state.value

    current[from] = toRaw
    current[to] = fromRaw
    state.value = current
  }

  /**
   * Splits a window into two if possible
   */
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

  /**
   * Inserts a new window into a specific location
   */
  function replace(location: WindowLocation, newState: Window) {
    state.value[location] = newState
  }

  return {
    windows: readonly(state),
    close,
    split,
    swap,
    replace,
  }
}
