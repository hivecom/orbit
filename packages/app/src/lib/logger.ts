type LogType = "log" | "error" | "warn"

const logStack = new Map<number, string[]>()

let id = 0

function getId() {
  return id + 1
}

/**
 * Creates a simple logger instance exposing `log` `error` and `warn` methods
 *
 * @param stackKey Numbered key of the stack. By default it's the zero-indexed
 * count of all active loggers.
 */
export function createLogger(stackKey: number = getId()) {
  function logger(type: LogType, scope: string, message: string, silent: boolean = true) {
    const text = `[${scope}] ${message}`

    const stackMessages = logStack.get(stackKey) ?? []
    stackMessages.push(text)
    logStack.set(stackKey, stackMessages)

    if (silent === false) {
      console[type](text)
    }
  }

  return {
    log: logger.bind(null, "log"),
    warn: logger.bind(null, "warn"),
    err: logger.bind(null, "error"),
  }
}

declare global {
  /**
   * Logs the specific stack and if no key is provided, logs and dumps _all_ stacks.
   *
   * @param stackKey
   */
  function popLogs(stackKey?: number): void
}

if (import.meta.env.DEV) {
  globalThis.popLogs = function (stackKey) {
    let str = ""
    if (!stackKey) {
      Array.from(logStack.entries()).forEach(([index, messages]) => {
        if (!messages || messages.length === 0) return

        str += `\n>>> Stack ${index} <<<\n`
        str += `${messages.join("\n")}\n`
      })
    } else {
      const stack = logStack.get(stackKey)
      if (!stack || stack.length === 0) return

      str += `\n>>> Stack ${stackKey} <<<\n`
      str += `${stack.join("\n")}`
    }
  }
}
