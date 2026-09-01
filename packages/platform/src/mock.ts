import type { Platform } from "./types"

/**
 * Mimics a fully implemented platform object with noop methods. Used in tests
 * or when initializing empty platforms before implementation
 */
export function createMockPlatform(target: Platform["target"]): Platform {
  return {
    target,
    notifications: {
      async requestPermission() {
        return Promise.resolve(true)
      },
      notify() {
        return Promise.resolve()
      },
    },
    tray: {
      async setTitle(title) {
        void title
      },
      async setBadgeCount(count) {
        void count
      },
      async addBadgeAlert() {
        void null
      },
      async removeBadge() {
        void null
      },
    },
    audioDevices: {
      enumerate: async () => [],
      onChange(listener) {
        void listener
        return () => void null
      },
    },
    deepLinks:
      target === "web"
        ? null
        : {
            onOpen(listener) {
              void listener
              return () => void null
            },
          },
    fileTransfer: {
      download({ url, filename }) {
        void url
        void filename
        return Promise.resolve()
      },
    },
    dns:
      target === "web"
        ? null
        : {
            resolveSrv(server) {
              void server
              return Promise.resolve([])
            },
          },
    historyCache: {
      seed(target: string, limit: number) {
        void target
        void limit
        return Promise.resolve([])
      },
      pageBefore(target, beforeMsgid, limit) {
        void target
        void beforeMsgid
        void limit
        return Promise.resolve([])
      },
      pageAfter(target, afterMsgid, limit) {
        void target
        void afterMsgid
        void limit
        return Promise.resolve([])
      },
      upsert(messages) {
        void messages
        return Promise.resolve()
      },
      markRedacted(msgid) {
        void msgid
        return Promise.resolve()
      },
      bufferStats: () => Promise.resolve([]),
      prune(target, keepCount) {
        void target
        void keepCount
        return Promise.resolve()
      },
      export(target) {
        void target
        return Promise.resolve([])
      },
      clear: () => Promise.resolve(),
    },
  }
}
