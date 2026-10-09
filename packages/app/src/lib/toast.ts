import { pushToast } from "@dolanske/vui"
import { markRaw } from "vue"
import ToastError from "../components/toast/ToastError.vue"
import type { OrbitError } from "core/types"

export function toastError(error: OrbitError, title = "Error leaving channel") {
  // TODO: bug in vui doesnt allow passing just options, so add empty string
  pushToast("", {
    body: markRaw(ToastError),
    bodyProps: {
      title,
      error,
    },
  })
}
