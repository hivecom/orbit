import { describe, expect, it } from "vite-plus/test"
import { createOrbitApp } from "../src/index.ts"
import { createWebPlatform } from "platform/src/web.ts"
import { PLATFORM_KEY } from "platform"
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
    expect(selector.get("[data-test-target]")).not.toBeNull()
  })
})
