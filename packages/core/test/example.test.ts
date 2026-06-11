import { mount } from "@vue/test-utils"
import RouteMain from "../src/router/views/RouteMain.vue"
import { expect, test } from "vite-plus/test"

test("Renders the landing page", () => {
  const wrapper = mount(RouteMain)
  const title = wrapper.get("h2")
  expect(title.text()).toBe("Orbit")
})
