import fs from "node:fs"
import path from "node:path"

fs.rmSync(path.join(import.meta.dirname, "./pkg/.gitignore"))
