import { defineConfig } from "vite-plus"
import vue from "@vitejs/plugin-vue"

// https://vite.dev/config/
// @ts-ignore
export default defineConfig({
  // Pages serves the prototype branch out of /prototype/ on orbit.talk, so CD
  // sets this for that build only. Everything else builds at the root.
  base: process.env.ORBIT_WEB_BASE ?? "/",

  plugins: [vue()],
  server: { port: 3000 },
})
