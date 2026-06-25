# Application core

The heart of the Orbit client. Every build target - web, desktop, and the embeddable widget - is served from this package with only the platform adapter swapped out. Application logic lives here.

UI is built with [`@dolanske/vui`](https://github.com/dolanske/vui); state uses Pinia.

## The Platform Seam

Core never imports `@tauri-apps/api` or reaches into raw `navigator.*`. Instead it declares capability ports and consumes them through an injected `Platform`:

```ts
import { usePlatform } from "app"

const platform = usePlatform()
await platform.notifications.requestPermission()
```

## Commands

```sh
vp test          # run the test suite (test/)
```

Run from the workspace root with `vp lint` / `vp run -r test` / `vp run -r build` to include every package.

See the spec's [Application Seams](https://github.com/hivecom/orbit-spec/blob/main/spec/05-infrastructure/04-application-seams.md) for the full design.
