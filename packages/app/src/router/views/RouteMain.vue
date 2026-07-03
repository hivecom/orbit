<script setup lang="ts">
import { Flex } from "@dolanske/vui"
import ServerConnectDialog from "../../components/dialogs/ServerConnectDialog.vue"
import UsernameDialog from "../../components/dialogs/UsernameDialog.vue"
import { useIrcStore } from "../../stores/irc.ts"
import { onBeforeMount } from "vue"

const irc = useIrcStore()
// initial application routing

onBeforeMount(() => {
  if (irc.servers.length > 0) {
    // Redirect to chat
  }
})
</script>

<template>
  <Flex x-center y-center column class="h-100">
    <div class="container-xs">
      <ServerConnectDialog>
        <template #stepper> </template>
      </ServerConnectDialog>
      <div class="mb-m"></div>
      <UsernameDialog />

      <Flex x-center expand class="mt-m">
        <RouterLink :to="{ path: '/wm' }">Open chat</RouterLink>
      </Flex>
    </div>
  </Flex>
</template>
