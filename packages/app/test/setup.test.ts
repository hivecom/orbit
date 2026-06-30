import { describe, expect, it } from "vite-plus/test"
import { createOrbitApp } from "../src/index.ts"
import { createWebPlatform } from "connector/src/web.ts"
import { PLATFORM_KEY } from "connector"
import { mount } from "@vue/test-utils"
import TestApp from "./fixtures/TestApp.vue"

describe("Setup Orbit application", () => {
  it("should create an app instance with a web platform adapter", () => {
    const platform = createWebPlatform()
    const app = createOrbitApp(TestApp, platform)
    expect(app._context.provides[PLATFORM_KEY]).toBe(platform)
  })

  it("should should be available within vue components when using usePlatform", () => {
    const selector = mount(TestApp, {
      global: {
        provide: {
          [PLATFORM_KEY]: createWebPlatform(),
        },
      },
    })

    const target = selector.get("[data-test-target]")
    expect(target.attributes("data-test-target")).toBe("web")
  })
})
