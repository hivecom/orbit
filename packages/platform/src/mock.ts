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
      notify({ title, body, icon }) {
        return new Notification(title, { body, icon })
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
    deepLinks: null,
    fileTransfer: {
      download({ url, filename }) {
        void url
        void filename
        return Promise.resolve()
      },
    },
    dns: null,
    historyCache: null,
  }
}
