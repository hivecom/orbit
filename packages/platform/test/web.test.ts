import { describe, expect, it, vi } from "vite-plus/test"
import { createWebPlatform } from "../src/index.ts"

describe("web platform adapter", () => {
  it("reports the web target", () => {
    expect(createWebPlatform().target).toBe("web")
  })

  it("nulls capabilities that require a native shell", () => {
    const platform = createWebPlatform()
    expect(platform.tray).toBeNull()
    expect(platform.deepLinks).toBeNull()
    expect(platform.dns).toBeNull()
  })

  it("provides browser-backed capabilities", () => {
    const platform = createWebPlatform()
    expect(platform.notifications).not.toBeNull()
    expect(platform.audioDevices).not.toBeNull()
    expect(platform.fileTransfer).not.toBeNull()
  })

  it("downloads via a transient anchor (jsdom)", async () => {
    const clicked: { href: string; download: string }[] = []
    const click = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(function mockClick(this: HTMLAnchorElement) {
      clicked.push({ href: this.href, download: this.download })
    })
    try {
      await createWebPlatform().fileTransfer.download({
        url: "https://example.org/file.png",
        filename: "file.png",
      })
      expect(clicked).toEqual([{ href: "https://example.org/file.png", download: "file.png" }])
      expect(document.querySelector("a")).toBeNull()
    } finally {
      click.mockRestore()
    }
  })
})
