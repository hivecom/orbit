import { ChannelRole, type Server } from "core-wasm"

export function truncate(text: string, limit: number, append?: string) {
  return text.substring(0, limit) + append
}

export function getServerInitials(metadata: Server["metadata"]) {
  return (metadata.name?.slice(0, 2) ?? metadata.address.startsWith("wss://")) ? metadata.address.slice(6, 8).toUpperCase() : metadata.address.slice(0, 2)
}

export function capitalize(text: string) {
  return text.substring(0, 1).toUpperCase() + text.substring(1)
}

export function getUserRole(role: ChannelRole, omitRegular?: boolean) {
  if (omitRegular && role === ChannelRole.Regular) return null
  switch (role) {
    case ChannelRole.Admin:
      return { label: "Admin", badgeType: "danger" }
    case ChannelRole.HalfOperator:
      return { label: "1/2 Op", badgeType: "warning" }
    case ChannelRole.Operator:
      return { label: "Op", badgeType: "warning" }
    case ChannelRole.Owner:
      return { label: "Owner", badgeType: "note" }
    case ChannelRole.Voice:
      return { label: "Voice", badgeType: "info" }
    case ChannelRole.Regular:
      return { label: "Regular", badgeType: "neutral" }
  }
}
