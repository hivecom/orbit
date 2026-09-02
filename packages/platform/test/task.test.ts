import { describe, it, expect, vi, beforeEach, afterEach } from "vite-plus/test"
import { TaskQueue } from "../src/task.ts"

const createPromise = (timeout: number, message?: string) =>
  new Promise((resolve) => {
    setTimeout(() => {
      resolve(message)
    }, timeout)
  })

describe("Task queing system", () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it("Should run tasks sequentially", async () => {
    const tasks = new TaskQueue()

    const spyOne = vi.fn(() => createPromise(100, "one"))
    const spyTwo = vi.fn(() => createPromise(50, "two"))
    const spyThree = vi.fn(() => createPromise(20, "three"))

    const p1 = tasks.queue(spyOne)
    const p2 = tasks.queue(spyTwo)
    const p3 = tasks.queue(spyThree)

    // Force next frame because task is not executed instantl;y
    await Promise.resolve()

    // Task 1 starts immediately, but tasks 2 & 3 must NOT have been called yet
    expect(spyOne).toHaveBeenCalledTimes(1)
    expect(spyTwo).not.toHaveBeenCalled()
    expect(spyThree).not.toHaveBeenCalled()

    // Task 1 should be done now
    await vi.advanceTimersByTimeAsync(100)
    await expect(p1).resolves.toBe("one")

    // Now Task 2 should have started, but NOT Task 3
    expect(spyTwo).toHaveBeenCalledTimes(1)
    expect(spyThree).not.toHaveBeenCalled()

    // Task 2 finishes
    await vi.advanceTimersByTimeAsync(50)
    await expect(p2).resolves.toBe("two")

    // Task 3 started
    expect(spyThree).toHaveBeenCalledTimes(1)

    await vi.advanceTimersByTimeAsync(20)
    await expect(p3).resolves.toBe("three")

    // Check for correct order
    expect(spyOne).toHaveBeenCalledBefore(spyTwo)
    expect(spyTwo).toHaveBeenCalledBefore(spyThree)
  })

  it("Should continue executing remaining tasks even if one fails", async () => {
    const tasks = new TaskQueue()

    const spyFail = vi.fn(async () => {
      await createPromise(50)
      throw new Error("Task failed")
    })
    const spySuccess = vi.fn(() => createPromise(30, "success"))

    const pFail = tasks.queue(spyFail)
    const pSuccess = tasks.queue(spySuccess)

    await vi.advanceTimersByTimeAsync(50)
    await expect(pFail).rejects.toThrow("Task failed")

    await vi.advanceTimersByTimeAsync(30)
    await expect(pSuccess).resolves.toBe("success")

    expect(spySuccess).toHaveBeenCalledTimes(1)
  })
})
