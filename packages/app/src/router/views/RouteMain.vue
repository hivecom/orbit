<script setup lang="ts">
import { Flex } from "@dolanske/vui"
import ServerConnectDialog from "../../components/dialogs/ServerConnectDialog.vue"
// import { useIrcStore } from "../../stores/irc.ts"
import { onMounted, ref } from "vue"
import { useRouter } from "vue-router"
import UsernameDialog from "../../components/dialogs/UsernameDialog.vue"
import Stepper from "../../components/shared/Stepper.vue"
import { useUserStore } from "../../stores/user.ts"
// import { useIrcStore } from "../../stores/irc.ts"

const router = useRouter()
const user = useUserStore()

// First time open state sync
const step = ref<"username" | "server">("username")

onMounted(() => {
  if (user.me.accountName && user.me.displayName) {
    step.value = "server"
  }
})

function redirectToChat() {
  router.push("/wm")
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
