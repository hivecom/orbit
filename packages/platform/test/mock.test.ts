import { describe, expect, it } from "vite-plus/test"
import { createMockPlatform } from "../src/mock.ts"

describe("Mock platforms", () => {
  it("should contain all non-nullable fields", () => {
    const mocked = createMockPlatform("web")

    expect(mocked.target).toBeDefined()

    expect(mocked.notifications.requestPermission).toBeDefined()
    expect(mocked.notifications.notify).toBeDefined()

    expect(mocked.tray).toBeDefined()
    expect(mocked.tray.setTitle).toBeDefined()
    expect(mocked.tray.setBadgeCount).toBeDefined()
    expect(mocked.tray.addBadgeAlert).toBeDefined()
    expect(mocked.tray.removeBadge).toBeDefined()

    expect(mocked.audioDevices).toBeDefined()
    expect(mocked.audioDevices.enumerate).toBeDefined()
    expect(mocked.audioDevices.onChange).toBeDefined()

    expect(mocked.fileTransfer).toBeDefined()
    expect(mocked.fileTransfer.download).toBeDefined()

    expect(mocked.historyCache.seed).toBeDefined()
    expect(mocked.historyCache.pageBefore).toBeDefined()
    expect(mocked.historyCache.pageAfter).toBeDefined()
    expect(mocked.historyCache.upsert).toBeDefined()
    expect(mocked.historyCache.markRedacted).toBeDefined()
    expect(mocked.historyCache.bufferStats).toBeDefined()
    expect(mocked.historyCache.prune).toBeDefined()
    expect(mocked.historyCache.export).toBeDefined()
    expect(mocked.historyCache.clear).toBeDefined()
  })

  it("Should contain nullable fields on desktop", () => {
    const mocked = createMockPlatform("desktop")
    expect(mocked.deepLinks).not.toBeNull()
    expect(mocked.dns).not.toBeNull()
  })

  it("Should not contain nullable fields on web", () => {
    const mocked = createMockPlatform("web")
    expect(mocked.deepLinks).toBeNull()
    expect(mocked.dns).toBeNull()
  })
})
