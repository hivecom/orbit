<script setup lang="ts">
import type { WindowChat } from "../../lib/windows"

// TODO: if no chat is active, list available chats for a server. We know that
// because channelId will be "__unspecified"

const props = defineProps<WindowChat>()
</script>

<template>
  <div></div>
</template>
