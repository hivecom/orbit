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

The workspace uses **[vite-plus](https://github.com/vitejs/vite-plus)** (`vp`) as the task runner. Requires Node >= 24.

```sh
vp i                    # initialize the project
vp create               # add a new package/app to the monorepo
vp run test             # run every test suite

vp run dev              # start the apps/web dev server
vp run build            # build the apps/web application
```

Use `vp test` inside an individual package when you want that package's local Vite/Vitest config, for example `packages/app` or `packages/platform`. From the workspace root, use `vp run -r test` so each package runs under its own test setup.

## License

[AGPL-3.0](LICENSE).
