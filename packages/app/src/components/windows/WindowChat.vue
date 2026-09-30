<script setup lang="ts">
import { MessageType } from "core-wasm"
import { type WindowAndLocation, type WindowChat } from "../../lib/windows"
import { useIrcStore } from "../../stores/irc"
import Composer from "../shared/composer/Composer.vue"
import { Accordion, DropdownItem, Flex, Grid } from "@dolanske/vui"
import { computed, nextTick, onMounted, ref, useTemplateRef } from "vue"
import { useEventListener, useThrottleFn } from "@vueuse/core"
import { IRC_UNKNOWN_CHANNEL, IRC_UNKNOWN_SERVER } from "../../lib/constants.ts"
import { useIRCJoinChannel } from "../../composables/useIRCJoinChannel.ts"
import { useDateFormatter } from "../../lib/date.ts"
import { useConfigStore } from "../../stores/config.ts"

const props = defineProps<WindowAndLocation<WindowChat>>()
const irc = useIrcStore()
const config = useConfigStore()

const format = useDateFormatter()

const messages = computed(() => irc.getChannelMessages(props.serverId, props.channelId))
// const state = computed(() => irc.getServerState(props.serverId))
// const channels = computed(() => irc.getServerChannels(props.serverId))
const channel = computed(() => irc.getServerChannel(props.serverId, props.channelId))

function sendMessage(message: string) {
  if (!channel.value) return
  channel.value.handler.send_message(message)
}

// Chat width & position config
const chatPositionStyle = computed(() => ({
  width: config.options.appearance_chat_width + "%",
  ...(config.options.appearance_chat_center_chat && { margin: "auto" }),
}))

const composerPositionStyle = computed(() => {
  // Same as with chat, except composer width is only modified, if chat is centered
  if (!config.options.appearance_chat_center_chat) return {}
  return {
    width: config.options.appearance_chat_width + "%",
    margin: "auto",
  }
})

// Automatic message fetching on scroll
const scrollLoading = ref(false)
const scrollContainer = useTemplateRef("chatScrollContainer")
const SCROLL_THRESHOLD = 200

const debouncedScrollCheck = useThrottleFn(async (event: Event) => {
  const target = event.target as HTMLElement
  if (target.scrollTop <= SCROLL_THRESHOLD && !scrollLoading.value) {
    scrollLoading.value = true

    const prevHeight = target.scrollHeight

    await irc.requestScrollback(props.serverId, props.channelId)
    await nextTick()

    // Adjust scroll position, otherwise we'll be triggering the fetch constantly
    const newHeight = target.scrollHeight
    target.scrollTop += newHeight - prevHeight

    scrollLoading.value = false
  }
}, 100)

useEventListener(scrollContainer, "scroll", debouncedScrollCheck)

// If user opens a window on a server where they haven't joined any channels, we
// must give them a choice to join one
const { join, loading: loadingChannel } = useIRCJoinChannel()

const accordions = useTemplateRef("accordionRef")

onMounted(() => {
  // For whatever reason it will open the channel list partially unless we add a
  // short delay. My guess is that the channel list isn't rendered yet but that
  // behavior is still weird.
  setTimeout(() => {
    if (props.serverId !== IRC_UNKNOWN_SERVER) {
      const index = irc.serversWithChannels.findIndex((item) => item.id === props.serverId)
      accordions.value?.[index]?.open()
    } else {
      accordions.value?.[0]?.open()
    }
  }, 50)
})
</script>

<template>
  <div class="o-window-chat">
    <div class="o-window-meta" v-if="props.channelId !== IRC_UNKNOWN_CHANNEL">
      <p>{{ props.channelId }}</p>
    </div>
    <div class="o-channel-list" v-if="props.channelId === IRC_UNKNOWN_CHANNEL || props.serverId === IRC_UNKNOWN_SERVER">
      <!-- <pre>
        {{ props }}
      </pre> -->
      <!-- 
    Render all servers
      1. if server id is -1 we open first accordion
      2. if server is id real, we open accordion of that server -->
      <Flex column x-center y-center class="h-100">
        <div class="container-s">
          <Accordion card ref="accordionRef" v-for="server in irc.serversWithChannels" :data-server="server.id" :key="server.id" :label="server.metadata.name ?? server.metadata.address">
            <Grid :columns="4">
              <DropdownItem :disabled="loadingChannel" v-for="channel in server.groupedChannels.joined" :key="channel.data.metadata.name" @click="join(server.id, channel.data.metadata.name, { forceLocation: props.location })">
                {{ channel.data.metadata.name }}
              </DropdownItem>
              <DropdownItem class="lighter" :disabled="loadingChannel" v-for="channel in server.groupedChannels.available" :key="channel.name" @click="join(server.id, channel.name, { forceLocation: props.location })">
                {{ channel.name }}
              </DropdownItem>
            </Grid>
          </Accordion>
          <!-- </AccordionGroup> -->
        </div>
      </Flex>
    </div>
    <div class="o-table-wrap" v-else :style="chatPositionStyle">
      <div class="o-table-scroll-container" ref="chatScrollContainer">
        <table class="o-msg-table">
          <tr v-for="message in messages" :key="message.metadata.msgid">
            <td class="msg-timestamp" v-if="config.options.appearance_chat_timestamps_enabled">{{ format.chatTimestamp(message.metadata.server_time) }}</td>
            <td class="msg-username">{{ message.metadata.user }}</td>
            <td class="msg-content" :class="{ status: message.metadata.message_type !== MessageType.Privmsg }">
              <template v-if="message.metadata.message_type === MessageType.Privmsg">{{ message.text?.content }} </template>
              <template v-else-if="message.metadata.message_type === MessageType.Join"> joined </template>
              <template v-else-if="message.metadata.message_type === MessageType.Part"> left </template>
              <template v-else> quit </template>
            </td>
          </tr>
        </table>
        <a id="scroll-anchor"></a>
      </div>
    </div>

    <div class="o-window-composer" v-if="props.channelId !== IRC_UNKNOWN_CHANNEL" :style="composerPositionStyle">
      <Composer @send="sendMessage" :placeholder="`Message ${props.channelId}`" />
    </div>
  </div>
</template>
<style scoped>
.o-window-chat {
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;

  .o-window-meta {
    display: flex;
    align-items: center;
    padding-inline: var(--space-s);
    border-bottom: 1px solid var(--color-border);
    height: 44px;
    background: var(--color-bg-lowered);
    z-index: 5;
  }

  .o-window-composer {
    position: sticky;
    bottom: 0;
  }

  .o-channel-list {
    flex: 1;
  }

  .o-table-wrap {
    flex: 1;
    position: relative;

    .o-table-scroll-container {
      overflow-anchor: none;
      position: absolute;
      bottom: 0;
      left: 0;
      right: 0;
      max-height: 100%;
      padding-bottom: var(--space-s);
      overflow-y: auto;

      /* FIXME: this doesnt't automatically scroll when window is rendered */
      #scroll-anchor {
        overflow-anchor: auto;
        height: 1px;
      }

      .o-msg-table {
        table-layout: auto;

        td {
          font-family: var(--font-mono);
          padding-inline: var(--space-xxxs);
          padding-block: var(--space-xxs);
          min-width: unset;
          border-radius: 0 !important;
          border-left: none;
          border-right: none;
          font-size: var(--font-size-s);
          border-bottom: none;

          &.msg-timestamp,
          &.msg-username {
            color: var(--color-text-lighter);
            white-space: nowrap;
          }

          &.msg-username {
            color: var(--color-text-light);
            padding-right: var(--space-m);
            padding-left: var(--space-xxs);
          }

          &.status {
            color: var(--color-text-lighter);
            font-style: italic;
          }

          &:nth-child(1) {
            padding-left: var(--space-m);
          }

          &:nth-child(3) {
            width: 100%;
          }
        }
      }
    }
  }
}
</style>
