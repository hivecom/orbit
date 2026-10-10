import fs from "node:fs"
import path from "node:path"

const pkg = path.join(import.meta.dirname, "pkg")

fs.rmSync(path.join(pkg, ".gitignore"))

// The closure shims in InitOutput are named after mangled Rust symbols, so they
// change with the toolchain even when the API doesn't. Nothing in TS calls them,
// and dropping them keeps the committed d.ts the same on every machine.
const dts = path.join(pkg, "core_wasm.d.ts")
const source = fs.readFileSync(dts, "utf8")

fs.writeFileSync(dts, source.replace(/^ *readonly \w*convert__closures\w*: .*\n/gm, ""))
