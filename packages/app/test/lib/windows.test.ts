import { afterEach, beforeEach, describe, expect, it, vi } from "vite-plus/test"
import { nextTick } from "vue"
import type { Window } from "../../src/lib/windows"
import { createPinia, setActivePinia } from "pinia"

const PATH = "../../src/lib/windows"

// Mock setup
const mocks = vi.hoisted(() => ({
  irc: { serverData: new Map<number, { id: number }>() },
  flags: { WINDOWS_PERSIST: false },
  router: {
    currentRoute: { value: { path: "/" } },
    push: vi.fn(async (path: string) => {
      mocks.router.currentRoute.value.path = path
    }),
  },
}))

vi.mock("../../src/stores/irc", () => ({ useIrcStore: () => mocks.irc }))
vi.mock("../../src/flags", () => ({ flags: mocks.flags }))
vi.mock("vue-router", () => ({ useRouter: () => mocks.router }))

// Helpers
const chat = (serverId = 1, channelId = "#general"): Window => ({ type: "chat", serverId, channelId })
const voice = (channelId = "voice-1"): Window => ({ type: "voice", channelId })
const empty = (): Window => ({ type: "empty" })

function addServer(id: number) {
  mocks.irc.serverData.set(id, { id })
}

// windows.ts keeps module-level state (`windows`, `focusedWindow`, `setupDone`),
// so every test that touches the manager needs a fresh copy of the module.
async function loadModule() {
  vi.resetModules()
  return await import(PATH)
}

function createStorageMock(): Storage {
  let store = new Map<string, string>()
  return {
    get length() {
      return store.size
    },
    key: (i) => Array.from(store.keys())[i] ?? null,
    getItem: (k) => store.get(k) ?? null,
    setItem: (k, v) => void store.set(k, String(v)),
    removeItem: (k) => void store.delete(k),
    clear: () => void (store = new Map()),
  }
}

let storage: Storage

beforeEach(() => {
  setActivePinia(createPinia())

  storage = createStorageMock()
  vi.stubGlobal("localStorage", storage)

  mocks.irc.serverData.clear()
  mocks.flags.WINDOWS_PERSIST = false
  mocks.router.currentRoute.value.path = "/"
  mocks.router.push.mockClear()
})

afterEach(() => {
  vi.unstubAllGlobals()
})

// Pure functions which are part of the composable API are omitted
// Functions that read the (mocked) irc store
describe("sanitizeState", () => {
  it("replaces chat windows for unknown servers with empty", async () => {
    const { sanitizeState } = await import(PATH)
    addServer(1)

    const result = sanitizeState({ l: chat(1), r: chat(99) })

    expect(result).toEqual({ l: chat(1), r: empty() })
  })
})

describe("deserializeState", () => {
  it("returns undefined for invalid JSON, unknown locations and empty objects", async () => {
    const { deserializeState } = await import(PATH)

    expect(deserializeState("not json")).toBeUndefined()
    expect(deserializeState("{}")).toBeUndefined()
    expect(deserializeState(JSON.stringify({ nope: empty() }))).toBeUndefined()
    expect(deserializeState(JSON.stringify({ f: { type: "chat" } }))).toBeUndefined()
    expect(deserializeState(JSON.stringify({ f: empty() }))).toEqual({ f: empty() })
  })
})

describe("useWindowManager", () => {
  it("starts on the first server and splits into a focused empty pane, then swaps them", async () => {
    addServer(7)
    const { useWindowManager } = await loadModule()
    const wm = useWindowManager()

    wm.setup()
    expect(wm.windows.value.f).toMatchObject({ type: "chat", serverId: 7 })

    wm.split("f")

    expect(wm.windows.value.f).toBeUndefined()
    expect(wm.windows.value.l).toMatchObject({ type: "chat", serverId: 7 })
    expect(wm.windows.value.r).toEqual(empty())
    expect(wm.focusedWindow.value?.location).toBe("r")

    wm.swap("l", "r")
    expect(wm.windows.value.r).toMatchObject({ type: "chat", serverId: 7 })
    expect(wm.windows.value.l).toEqual(empty())
  })

  it("closes a window and never closes f", async () => {
    addServer(7)
    const { useWindowManager } = await loadModule()
    const wm = useWindowManager()
    wm.setup()
    wm.split("f")
    expect(wm.windows.value.l).toBeDefined()
    expect(wm.windows.value.r).toBeDefined()

    wm.close("r")
    expect(wm.windows.value.f).toBeDefined()
    expect(wm.windows.value.l).toBeUndefined()

    wm.close("f")
    expect(wm.windows.value.f).toBeDefined()
  })

  it("navigates to /wm when replacing from another route", async () => {
    const { useWindowManager } = await loadModule()
    const wm = useWindowManager()
    wm.setup()

    await wm.replace("f", voice("v"))

    expect(mocks.router.push).toHaveBeenCalledWith("/wm")
    expect(wm.windows.value.f).toEqual(voice("v"))
  })

  it("resets state back to single f window", async () => {
    addServer(7)
    const { useWindowManager } = await loadModule()
    const wm = useWindowManager()
    wm.setup()
    wm.split("f")
    expect(wm.windows.value.l).toBeDefined()
    expect(wm.windows.value.r).toBeDefined()

    wm.reset()
    expect(wm.windows.value.l).toBeUndefined()
    expect(wm.windows.value.r).toBeUndefined()
    expect(wm.windows.value.f).toBeDefined()
  })
})

describe("persistence", () => {
  it("writes state to localStorage when WINDOWS_PERSIST is on", async () => {
    mocks.flags.WINDOWS_PERSIST = true
    const { useWindowManager } = await loadModule()
    const wm = useWindowManager()
    wm.setup()

    await wm.replace("f", voice("persisted"))
    await nextTick() // let the deep watcher flush

    const stored = JSON.parse(localStorage.getItem("o-wm-state-v2") ?? "{}")
    expect(stored.f).toEqual(voice("persisted"))
  })
})
