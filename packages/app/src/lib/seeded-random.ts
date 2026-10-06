export function randomSeed(seed: string) {
  function alphabetPosition(text: string) {
    let result = ""
    for (let i = 0; i < text.length; i++) {
      const code = text.toUpperCase().charCodeAt(i)
      if (code > 64 && code < 91) result += code - 64
    }

    return Number(result.slice(0, result.length - 1))
  }

  let a = alphabetPosition(seed)

  let t = (a += 0x6d2b79f5)
  t = Math.imul(t ^ (t >>> 15), t | 1)
  t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296
}

const seedCache = new Map<string, number>()

export function seedRandomRange(min: number, max: number, seed: string) {
  if (seedCache.has(seed)) return seedCache.get(seed) as number
  min = Math.ceil(min)
  max = Math.floor(max)
  const result = Math.floor(randomSeed(seed) * (max - min + 1)) + min
  seedCache.set(seed, result)
  return result
}

export function randomRange(min: number, max: number) {
  min = Math.ceil(min)
  max = Math.floor(max)
  return Math.floor(Math.random() * (max - min + 1)) + min
}
