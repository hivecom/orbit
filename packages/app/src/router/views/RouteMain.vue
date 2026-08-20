<script setup lang="ts">
import { Flex } from "@dolanske/vui"
import ServerConnectDialog from "../../components/dialogs/ServerConnectDialog.vue"
// import { useIrcStore } from "../../stores/irc.ts"
import { onBeforeMount, onMounted, ref } from "vue"
import { useRouter } from "vue-router"
import UsernameDialog from "../../components/dialogs/UsernameDialog.vue"
import Stepper from "../../components/shared/Stepper.vue"
import type { Server } from "core-wasm"
import { serializeWindow } from "../../lib/windows.ts"
import { IRC_UNKNOWN_CHANNEL } from "../../lib/constants.ts"
import { useUserStore } from "../../stores/user.ts"
import { useIrcStore } from "../../stores/irc.ts"

const router = useRouter()
const irc = useIrcStore()
const user = useUserStore()

// onBeforeMount(() => {
//   if (irc.serverData.size > 0) {
//     router.replace({ name: "RouteWindowManager" })
//   }
// })

// First time open state sync
const step = ref<"username" | "server">("username")

onMounted(() => {
  console.log(user.me)
  if (user.me.accountName && user.me.displayName) {
    step.value = "server"
  }
})

function redirectToChat(state: Server) {
  router.push({
    name: "RouteWindowManager",
    params: {
      f: serializeWindow({
        type: "chat",
        serverId: state.id,
        channelId: IRC_UNKNOWN_CHANNEL,
      }),
    },
  })
}
</script>

<template>
  <Flex x-center y-center column class="h-100">
    <div class="container-xs">
      <UsernameDialog v-if="step === 'username'" @success="step = 'server'">
        <template #stepper>
          <Stepper :model-value="1" :steps="2" />
        </template>
      </UsernameDialog>
      <ServerConnectDialog v-else-if="step === 'server'" @success="redirectToChat" @error="">
        <template #stepper>
          <Stepper :model-value="2" :steps="2" />
        </template>
      </ServerConnectDialog>
    </div>
  </Flex>
</template>
