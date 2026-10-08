import { defineConfig } from "vite-plus"

export default defineConfig({
  staged: {
    "*": "vp check --fix",
  },
  fmt: {
    semi: false,
    printWidth: 320,
    ignorePatterns: ["packages/core/core-wasm/pkg/**"],
  },
  lint: {
    ignorePatterns: ["dist/**", "packages/core/core-wasm/pkg/**"],
    jsPlugins: [{ name: "vite-plus", specifier: "vite-plus/oxlint-plugin" }],
    rules: {
      "vite-plus/prefer-vite-plus-imports": "error",
    },
    options: {
      typeAware: true,
      typeCheck: true,
    },
  },
  run: {
    cache: true,
  },
  test: {
    exclude: ["**/node_modules/**", "**/.direnv/**"],
  },
})
