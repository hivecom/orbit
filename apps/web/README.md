# web

The web app / PWA entrypoint for Orbit. A thin Vite boot shell - all application logic lives in [`core`](../../packages/core). This entrypoint's only job is to create the Vue app, mount `OrbitApp`, and inject the web platform adapter.

The web adapter (`createWebPlatform` from [`platform`](../../packages/platform)) wires up browser-backed capabilities. On lack of capabilities, core degrades gracefully.

Widget mode requires no separate build: it is this app accessed with `?mode=widget`.

## Commands

```sh
vp dev           # Vite dev server
vp build         # build the web app / PWA (vue-tsc -b && vp build)
vp preview       # preview the production build
```

From the workspace root, `pnpm dev` runs this app (`vp run web#dev`).
