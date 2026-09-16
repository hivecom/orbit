export function toJSON<Expected>(obj: object, defaultValue = null): Expected | null {
  try {
    return JSON.parse(JSON.stringify(obj))
  } catch {
    return defaultValue
  }
}
