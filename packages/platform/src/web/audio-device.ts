import type { AudioDevice, AudioDevicePort } from "../types"

export function createAudioDevicePort(): AudioDevicePort {
  return {
    async enumerate() {
      if (!navigator.mediaDevices?.enumerateDevices) return []
      const devices = await navigator.mediaDevices.enumerateDevices()
      return devices
        .filter((device) => device.kind === "audioinput" || device.kind === "audiooutput")
        .map<AudioDevice>((device) => ({
          id: device.deviceId,
          label: device.label || "Unknown device",
          kind: device.kind === "audioinput" ? "input" : "output",
        }))
    },
    onChange(listener) {
      const target = navigator.mediaDevices
      if (!target) return () => {}
      target.addEventListener("devicechange", listener)
      return () => target.removeEventListener("devicechange", listener)
    },
  }
}
