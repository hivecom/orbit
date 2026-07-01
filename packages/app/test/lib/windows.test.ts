import { describe, expect, it } from "vite-plus/test"
import { deserializeState, deserializeWindow, getDefaultState, serializeState, serializeWindow, WindowChat, WindowEmpty, WindowState, WindowVoice } from "../../src/lib/windows.ts"

// TODO: test the composable itself - requires jsdom context setup because it uses URL and localStorage

const chatWindow: WindowChat = {
  type: "chat",
  serverId: "asd-123",
  channelId: "xxx-12",
}

const voiceWindow: WindowVoice = {
  type: "voice",
  channelId: "asd-123",
}

const emptyWindow: WindowEmpty = {
  type: "empty",
}

const chatWindowSerialized = "c:asd-123:xxx-12"
const voiceWindowSerialized = "v:asd-123"
const emptyWindowSerialized = "e"

const windowState: WindowState = {
  l: chatWindow,
  rt: voiceWindow,
  rb: emptyWindow,
}

const windowStateSerialized = "l:c:asd-123:xxx-12;rt:v:asd-123;rb:e"

describe("wm methods", () => {
  it("should serialize window object into URL string", () => {
    expect(serializeWindow(chatWindow)).toBe(chatWindowSerialized)
    expect(serializeWindow(voiceWindow)).toBe(voiceWindowSerialized)
    expect(serializeWindow(emptyWindow)).toBe(emptyWindowSerialized)
    // @ts-expect-error Test case empty object
    expect(serializeWindow({})).toBe("e")
    // @ts-expect-error Test case empty input
    expect(serializeWindow()).toBe("e")
  })

  it("should deserialize string into a window object", () => {
    expect(deserializeWindow(chatWindowSerialized)).toMatchObject(chatWindow)
    expect(deserializeWindow(voiceWindowSerialized)).toMatchObject(voiceWindow)
    expect(deserializeWindow(emptyWindowSerialized)).toMatchObject(emptyWindow)
    expect(deserializeWindow("asdadas")).toMatchObject(emptyWindow)
    // @ts-expect-error Test case empty input
    expect(deserializeWindow()).toMatchObject(emptyWindow)
  })

  it("should serialize entire window state into a URL parameter value", () => {
    expect(serializeState(windowState)).toBe(windowStateSerialized)
    expect(serializeState({})).toBe("f:e")
    // @ts-expect-error Test case empty input
    expect(serializeState()).toBe("f:e")
  })

  it("should deserialize entire URL state into a window state object", () => {
    expect(deserializeState(windowStateSerialized)).toMatchObject(windowState)
    expect(deserializeState("abce")).toMatchObject(getDefaultState())
    // @ts-expect-error Test case empty input
    expect(deserializeState()).toMatchObject(getDefaultState())
  })

  it("should return default state", () => {
    expect(getDefaultState()).toMatchObject({
      f: {
        type: "empty",
      },
    })
  })
})
