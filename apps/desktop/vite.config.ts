import { fileURLToPath, URL } from "node:url"

import { defineConfig } from "vite-plus"
import vue from "@vitejs/plugin-vue"
import vueDevTools from "vite-plugin-vue-devtools"
import { lazyPlugins } from "vite-plus"

// https://vite.dev/config/
// @ts-ignore
export default defineConfig({
  fmt: {
    semi: false,
    singleQuote: true,
  },
  // @ts-ignore
  plugins: lazyPlugins(() => [vue(), vueDevTools()]),
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
})
