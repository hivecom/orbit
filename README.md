# Orbit

Orbit is a modern communication platform built on IRC. For more detailed summary and complete specification, check out [orbit-spec](https://github.com/hivecom/orbit-spec) repo.

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

### UI conventions

We use Solar icons for ui iconography, specifically the `linear` icon set.

## License

[AGPL-3.0](LICENSE).
