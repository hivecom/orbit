import type { NotificationPort } from "../types"

export function createNotificationPort(): NotificationPort {
  return {
    async requestPermission() {
      if (!("Notification" in globalThis)) return false
      if (Notification.permission === "granted") return true
      if (Notification.permission === "denied") return false
      const result = await Notification.requestPermission()
      return result === "granted"
    },
    notify({ title, body, icon }) {
      if (!("Notification" in globalThis) || Notification.permission !== "granted") {
        return null
      }

      void new Notification(title, { body, icon })
    },
  }
}
