import type { Server } from "core-wasm"

export function truncate(text: string, limit: number, append?: string) {
  return text.substring(0, limit) + append
}

export function getServerInitials(metadata: Server["metadata"]) {
  return (metadata.name?.slice(0, 2) ?? metadata.address.startsWith("wss://")) ? metadata.address.slice(6, 8).toUpperCase() : metadata.address.slice(0, 2)
}
