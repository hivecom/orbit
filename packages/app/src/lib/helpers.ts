export function toJSON(obj: object): object | null {
  try {
    return JSON.parse(JSON.stringify(obj))
  } catch {
    return null
  }
}
