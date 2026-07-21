<script setup lang="ts">
import type { WindowChat } from "../../lib/windows"
import { useIrcStore } from "../../stores/irc"

// TODO: if no chat is active, list available chats for a server. We know that
// because channelId will be "__unspecified"

const props = defineProps<WindowChat>()

const irc = useIrcStore()
const state = irc.getServerState(props.serverId)
</script>

<template>
  <div>
    <pre>
      {{ props }}
    </pre>
    <pre>{{ state }}</pre>
  </div>
</template>
