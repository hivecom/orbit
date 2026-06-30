import { defineConfig } from "vite-plus"

// https://vite.dev/config/
export default defineConfig({
  test: {
    environment: "jsdom",
  },
})
