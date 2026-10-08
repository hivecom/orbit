import type { FileTransferPort } from "../types"

export function createFileTransferPort(): FileTransferPort {
  return {
    async download({ url, filename }) {
      const anchor = document.createElement("a")
      anchor.href = url
      anchor.download = filename
      anchor.rel = "noopener"
      document.body.append(anchor)
      anchor.click()
      anchor.remove()
    },
  }
}
