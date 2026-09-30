import { computed, ref, watch } from "vue"
import { useIrcStore } from "../stores/irc"
import { IRC_UNKNOWN_CHANNEL } from "./constants"
import { flags } from "../flags"
import { useRouter } from "vue-router"

export type WindowLocation = "f" | "l" | "r" | "lt" | "lb" | "rt" | "rb"

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

export interface SplitResult {
  state: WindowState
  // Location of the newly created, empty pane
  focus: WindowLocation
}

////////////////////////////////////////////////////////////////////////

const WIN_STORAGE_KEY = "o-wm-state-v2"
const WIN_LOCATIONS: WindowLocation[] = ["f", "l", "r", "lt", "lb", "rt", "rb"]

export function getDefaultState(): WindowState {
  const irc = useIrcStore()

  // FIXME: check this before pushing
  // NOTE: this depends on the irc store already being hydrated with a
  // server list at the moment this runs. If setup() below runs before
  // servers have loaded, this falls through to the empty window even
  // though servers exist, and nothing currently re-evaluates it
  // afterwards. init() is now idempotent-safe to call again, so if this
  // turns out to matter in practice, the fix is to call init() once more
  // when the server list first becomes non-empty.
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
    f: { type: "empty" },
  }
}

////////////////////////////////////////////////////////////////////////

function isWindow(value: unknown): value is Window {
  if (!value || typeof value !== "object") return false
  const v = value as Record<string, unknown>

  switch (v.type) {
    case "chat":
      return typeof v.serverId === "number" && typeof v.channelId === "string"
    case "voice":
      return typeof v.channelId === "string"
    case "empty":
      return true
    default:
      return false
  }
}

function isWindowState(value: unknown): value is WindowState {
  if (!value || typeof value !== "object") return false

  return Object.entries(value as Record<string, unknown>).every(([location, window]) => WIN_LOCATIONS.includes(location as WindowLocation) && isWindow(window))
}

// Deserializes stored windows from localStorage
export function deserializeState(raw: string): WindowState | undefined {
  try {
    const parsed = JSON.parse(raw)
    if (!isWindowState(parsed) || Object.keys(parsed).length === 0) return undefined
    return parsed
  } catch {
    return undefined
  }
}

// Checks whether current state matches with the servers we are connected to.
export function sanitizeState(state: WindowState): WindowState {
  const irc = useIrcStore()
  const sanitized: WindowState = {}

  for (const location of WIN_LOCATIONS) {
    const window = state[location]
    if (!window) continue

    if (window.type === "chat" && !irc.serverData.has(window.serverId)) {
      sanitized[location] = { type: "empty" }
      continue
    }

    sanitized[location] = window
  }

  return sanitized
}

export function loadInitialState(): WindowState {
  if (!flags.WINDOWS_PERSIST) return getDefaultState()

  try {
    const raw = localStorage.getItem(WIN_STORAGE_KEY)
    if (raw) {
      const parsed = deserializeState(raw)
      if (parsed) return sanitizeState(parsed)
    }
  } catch {
    // TODO: handle errors where localStorage is missing
  }

  return getDefaultState()
}

////////////////////////////////////////////////////////////////////////

// Closes a window at a location and cascades windows into proper place
export function applyClose(state: WindowState, location: WindowLocation): WindowState {
  if (!state[location] || location === "f") return state

  const next: WindowState = { ...state }
  delete next[location]

  switch (location) {
    case "lt":
      next.l = next.lb
      delete next.lb
      break
    case "lb":
      next.l = next.lt
      delete next.lt
      break
    case "rt":
      next.r = next.rb
      delete next.rb
      break
    case "rb":
      next.r = next.rt
      delete next.rt
      break
    case "l":
      if (next.r) return { f: next.r }
      next.l = next.rt
      next.r = next.rb
      delete next.rt
      delete next.rb
      break
    case "r":
      if (next.l) return { f: next.l }
      next.l = next.lt
      next.r = next.lb
      delete next.lt
      delete next.lb
      break
  }

  return next
}

// Swaps two window positions
export function applySwap(state: WindowState, from: WindowLocation, to: WindowLocation): WindowState {
  const next: WindowState = { ...state }
  const fromWindow = state[from]
  const toWindow = state[to]

  if (toWindow) next[from] = toWindow
  else delete next[from]

  if (fromWindow) next[to] = fromWindow
  else delete next[to]

  return next
}

// Splits a window into two. Moving the active window to the left and creating an empty window next to it
export function applySplit(state: WindowState, from: WindowLocation, newWindow?: Window): SplitResult | undefined {
  const next: WindowState = { ...state }

  switch (from) {
    case "f":
      next.l = next.f
      next.r = newWindow ?? { type: "empty" }
      delete next.f
      return { state: next, focus: "r" }
    case "l":
      next.lt = next.l
      next.lb = newWindow ?? { type: "empty" }
      delete next.l
      return { state: next, focus: "lb" }
    case "r":
      next.rt = next.r
      next.rb = newWindow ?? { type: "empty" }
      delete next.r
      return { state: next, focus: "rb" }
    default:
      // lt/lb/rt/rb are already as deep as the 4-pane layout goes.
      return undefined
  }
}

// Replaces a window with the provided one
export function applyReplace(state: WindowState, location: WindowLocation, content: Window): WindowState {
  return { ...state, [location]: content }
}

// Finds where a specific window object ended up after a transition by reference
function locate(state: WindowState, window: Window | undefined): WindowLocation | null {
  if (!window) return null
  for (const location of WIN_LOCATIONS) {
    if (state[location] === window) return location
  }
  return null
}

// Returns the closest empty location
function firstAvailableLocation(state: WindowState): WindowLocation | null {
  for (const location of WIN_LOCATIONS) {
    if (state[location]) return location
  }
  return null
}

/**
 * Search for an active window via its IDs.
 */
export function findById({ serverId, channelId }: { serverId?: number; channelId?: string } = {}) {
  if (!serverId && !channelId) return null

  let data: WindowAndLocation | null = null

  for (const [location, item] of Object.entries(windows.value)) {
    if (("serverId" in item && item.serverId === serverId) || ("channelId" in item && item.channelId === channelId)) {
      data = {
        ...item,
        location,
      } as WindowAndLocation
      break
    }
  }

  return data
}

////////////////////////////////////////////////////////////////////////

const windows = ref<WindowState>({})
const focusedWindow = ref<WindowAndLocation | null>(null)
const isEmpty = computed(() => Object.values(windows.value).filter((item) => item && item.type !== "empty").length === 0)
const windowsCount = computed(() => Object.keys(windows.value).length)

// Focuses the provided location. This means that if user clicks on a window, it
// will be placed in this location
export function setFocus(location: WindowLocation | null) {
  if (!location) {
    focusedWindow.value = null
    return
  }
  const window = windows.value[location]
  focusedWindow.value = window ? { ...window, location } : null
}

// Ensure that focused window is followed if it was mutated or fallback to default location
function followFocus(previousWindow: Window | undefined) {
  const followedLocation = locate(windows.value, previousWindow)
  setFocus(followedLocation ?? firstAvailableLocation(windows.value))
}

let setupDone = false

export function setupWindows() {
  if (setupDone) return

  setupDone = true

  windows.value = loadInitialState()
  setFocus(firstAvailableLocation(windows.value))

  watch(
    windows,
    (newWindows) => {
      if (!flags.WINDOWS_PERSIST) return
      try {
        localStorage.setItem(WIN_STORAGE_KEY, JSON.stringify(newWindows))
      } catch {
        // TODO: handle localStorage errors (if we care)
      }
    },
    { deep: true },
  )
}

export function useWindowManager() {
  const router = useRouter()

  /**
   * Closes a window at a location. The layout will automatically reflow
   */
  function close(location: WindowLocation) {
    const previous = focusedWindow.value ? windows.value[focusedWindow.value.location] : undefined
    windows.value = applyClose(windows.value, location)
    followFocus(previous)
  }

  /**
   * Swaps two windows
   */
  function swap(from: WindowLocation, to: WindowLocation) {
    windows.value = applySwap(windows.value, from, to)

    if (focusedWindow.value && (focusedWindow.value.location === from || focusedWindow.value.location === to)) {
      setFocus(focusedWindow.value.location)
    }
  }

  /**
   * Splits a window into two if possible
   */
  function split(from: WindowLocation, content?: Window) {
    // if (!content) return

    const result = applySplit(windows.value, from, content)

    if (!result) return

    windows.value = result.state
    setFocus(result.focus)
  }

  /**
   * Inserts a new window into a specific location. Fallsback to current focus,
   * then to `f`
   */
  async function replace(location: WindowLocation, content: Window) {
    const target = windows.value[location] ? location : (focusedWindow.value?.location ?? "f")

    if (router.currentRoute.value.path !== "/wm") {
      await router.push("/wm")
    }

    windows.value = applyReplace(windows.value, target, content)
    setFocus(target)
  }

  /**
   * Resets state to just a single window (chat with the first available
   * server, or empty if there are none).
   */
  function reset() {
    windows.value = getDefaultState()
    setFocus(firstAvailableLocation(windows.value))
  }

  return {
    windows,
    windowsCount,
    focusedWindow,
    isEmpty,
    close,
    split,
    swap,
    replace,
    reset,
    setup: setupWindows,
    setFocus,
  }
}
