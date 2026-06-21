import { describe, expect, it } from "vite-plus/test"
import { User } from "../src/api.ts"

describe("API tests", () => {
  it("should have the expected API methods", () => {
    expect(typeof User.uploadStart).toBe("function")
    expect(typeof User.upload).toBe("function")
    expect(typeof User.delete).toBe("function")
  })
})
