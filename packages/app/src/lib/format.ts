import type { Server } from "core-wasm"

export function truncate(text: string, limit: number, append?: string) {
  return text.substring(0, limit) + append
}

export function getServerInitials(metadata: Server["metadata"]) {
  return (metadata.name?.charAt(0) ?? metadata.address.startsWith("wss://")) ? metadata.address.charAt(6).toUpperCase() : metadata.address.charAt(0)
}
