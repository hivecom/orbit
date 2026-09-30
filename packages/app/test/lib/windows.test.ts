import { beforeEach, describe } from "vite-plus/test"
import { createPinia, setActivePinia } from "pinia"

// TODO: test everything

describe.skip("wm methods", () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })
})
