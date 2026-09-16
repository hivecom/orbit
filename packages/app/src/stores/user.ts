import { defineStore } from "pinia"
import { reactive, shallowRef, watchEffect } from "vue"
import { USER_STORAGE_KEY } from "../lib/constants"

/**
 * User related data
 */
export const useUserStore = defineStore("user", () => {
  const initialized = shallowRef(false)
  const me = reactive({
    accountName: "",
    displayName: "",
    password: "",
  })

  watchEffect(() => {
    localStorage.setItem(
      USER_STORAGE_KEY,
      JSON.stringify({
        accountName: me.accountName,
        displayName: me.displayName,
      }),
    )
  })

  /**
   * Every global store ships with an init function which is always called in
   * the `createOrbitApp` and nowhere else. Takes in the initial dataset
   * returned by the IRC/Depot/etc servers.
   */
  function init() {
    try {
      const raw = localStorage.getItem(USER_STORAGE_KEY)
      if (!raw) return
      const parsed = JSON.parse(raw)
      if (!parsed.accountName || !parsed.displayName) return
      Object.assign(me, parsed)
    } catch {}

    initialized.value = true
  }

  function signIn(accountName: string, displayName: string, password: string) {
    Object.assign(me, {
      accountName,
      displayName,
      password,
    })
  }

  return {
    init,
    initialized,
    me,
    signIn,
  }
})
