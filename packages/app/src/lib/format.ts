export function truncate(text: string, limit: number, append?: string) {
  return text.substring(0, limit) + append
}
