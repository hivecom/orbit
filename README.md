# Orbit

Orbit is a modern communication platform built on IRC. Not an IRC client - a polished client layer, a voice/video service, and a file storage gateway, orchestrated into one cohesive product over infrastructure communities have trusted for thirty years.

Everything your friend group needs from Discord, privately, on a $5 VPS. `docker compose up`.

This repository is the **client monorepo**: the Vue 3 application that runs as a web app, desktop shell, and embeddable widget. The full system design lives in the [orbit-spec](https://github.com/hivecom/orbit-spec) repository.

## Architecture

Orbit owns only the parts where product value lives - the clients, Satellite (voice/video), and Depot (file access / permissions). The IRC server, identity provider, and storage backend are stock and adopted.

| Component       | Role                                                                                |
| --------------- | ----------------------------------------------------------------------------------- |
| **Clients**     | Desktop + Mobile (Tauri), web app, embeddable widget - this repo.                   |
| **Uplink**      | Any stock IRCv3 server (Ergo is the reference). Text, history, presence, signaling. |
| **Satellite**   | Real-time voice, video, and screen sharing over LiveKit.                            |
| **Depot**       | Thin storage gateway over S3-compatible backends or local disk.                     |
| **Transponder** | Any OIDC provider (Keycloak, Authentik, Supabase). One login, verified everywhere.  |

## Development

The workspace uses **pnpm** with a dependency catalog and **[vite-plus](https://github.com/vitejs/vite-plus)** (`vp`) as the task runner. Requires Node >= 24.

```sh
pnpm install            # install workspace dependencies
pnpm dev                # web dev server (runs apps/web)
pnpm ready              # gate: vp check && vp run -r test && vp run -r build
```

Scoped and recursive commands:

```sh
vp run web#dev          # Vite dev server for the web app
vp run web#build        # build the web app / PWA
vp run -r test          # run every test suite
vp run -r build         # build every package and app
```

## Specification

The complete design spec - architecture, components, identity, clients, infrastructure, ADRs, and research tracks - lives in [orbit-spec](https://github.com/hivecom/orbit-spec). Start with the [Platform Comparison](https://github.com/hivecom/orbit-spec/blob/main/spec/01-architecture/05-platform-comparison.md) and [Design Philosophy](https://github.com/hivecom/orbit-spec/blob/main/spec/01-architecture/02-philosophy.md).

## License

[AGPL-3.0](LICENSE).
