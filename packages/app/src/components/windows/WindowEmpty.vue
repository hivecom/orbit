<script setup lang="ts">
import { Button } from "@dolanske/vui"
import { useWindowManager, type WindowLocation, type WindowType } from "../../lib/windows"
import { IRC_UNKNOWN_CHANNEL, IRC_UNKNOWN_SERVER } from "../../lib/constants"
import { onBeforeMount, onMounted } from "vue"

interface Props {
  location: WindowLocation
}

const props = defineProps<Props>()

// When button is clicked, the manager will open a new window in this location
const emit = defineEmits<{
  open: [type: WindowType]
}>()

const { replace } = useWindowManager()

// FIXME We only have irc chat now, so instantly redirect to chat
onMounted(() => {
  replace(props.location, {
    serverId: IRC_UNKNOWN_SERVER,
    type: "chat",
    channelId: IRC_UNKNOWN_CHANNEL,
  })
})
</script>

<template>
  <div>
    <Button>Open chat</Button>
    <Button>Open voice</Button>
  </div>
</template>
