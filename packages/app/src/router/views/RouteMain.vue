<script setup lang="ts">
import { Flex } from "@dolanske/vui"
import ServerConnectDialog from "../../components/dialogs/ServerConnectDialog.vue"
import UsernameDialog from "../../components/dialogs/UsernameDialog.vue"
</script>

<template>
  <Flex x-center y-center column class="h-100">
    <div class="container-s">
      <ServerConnectDialog />
      <div class="mb-m"></div>
      <UsernameDialog />
      <RouterLink :to="{ path: '/wm' }">Open chat</RouterLink>
    </div>
  </Flex>
</template>
