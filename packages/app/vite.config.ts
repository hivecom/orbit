import { defineConfig } from "vite-plus"
import vue from "@vitejs/plugin-vue"

// https://vite.dev/config/
// @ts-ignore
export default defineConfig({
  plugins: [vue()],
  test: {
    environment: "jsdom",
  },
})
