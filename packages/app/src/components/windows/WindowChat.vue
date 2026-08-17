<script setup lang="ts">
import { MessageType } from "core-wasm"
import { useWindowManager, type WindowChat, type WindowLocation } from "../../lib/windows"
import { useIrcStore } from "../../stores/irc"
import Composer from "../shared/composer/Composer.vue"
import { Button, DropdownItem, Flex, PopoutHover } from "@dolanske/vui"
import { IconInfoCircleLinear } from "@iconify-prerendered/vue-solar"
import { nextTick, ref, useTemplateRef } from "vue"
import { useEventListener, useThrottleFn } from "@vueuse/core"
import { IRC_UNKNOWN_CHANNEL } from "../../lib/constants.ts"

// TODO: Figure out connecting to specific channels and showing the one that's open (and replacing url state)

interface Props extends WindowChat {
  location: WindowLocation
}

const props = defineProps<Props>()
const irc = useIrcStore()

const window = useWindowManager()

// previous channel was `#orbit/testing`
const messages = irc.getChannelMessages(props.serverId, props.channelId)
const state = irc.getServerState(props.serverId)
const channels = irc.getServerChannels(props.serverId)
// const channel = irc.getServerChannel(props.serverId, props.channelId)

function sendMessage(message: string) {
  // TODO: must send to the channe
  console.log(message)
  // irc.serverChannel?.send_message(message)
}

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
const loadingChannel = ref(false)

async function join(channelId: string) {
  loadingChannel.value = true
  await irc.channelJoin(props.serverId, channelId)

  // User has chosen a channel so now we can replace the current window with the
  // actual channel messages
  window.replace(props.location, {
    serverId: props.serverId,
    type: props.type,
    channelId: props.channelId,
  })

  loadingChannel.value = false
}
</script>

<template>
  <div class="o-window-chat" v-if="state">
    <div class="o-window-meta">
      <PopoutHover>
        <template #trigger>
          <Flex y-center gap="xs">
            <IconInfoCircleLinear />
            <p>{{ state?.metadata.address }}</p>
          </Flex>
        </template>

        <pre class="m-s">
          {{ JSON.stringify(state, null, 2) }}
        </pre>
      </PopoutHover>
    </div>
    <div class="o-channel-list" v-if="props.channelId === IRC_UNKNOWN_CHANNEL">
      <Flex column x-center y-center>
        <DropdownItem :disabled="loadingChannel" v-for="channel in channels?.available" :key="channel.name" @click="join(channel.name)">
          {{ channel.name }}
        </DropdownItem>

        <pre>
          {{ channels }}
        </pre>
      </Flex>
    </div>
    <div class="o-table-wrap" v-else>
      <div class="o-table-scroll-container" ref="chatScrollContainer">
        <table class="o-message-table">
          <tr v-for="message in messages" :key="message.metadata.msgid">
            <td class="message-username">{{ message.metadata.user }}</td>
            <td class="message-content" :class="{ status: message.metadata.message_type !== MessageType.Privmsg }">
              <template v-if="message.metadata.message_type === MessageType.Privmsg">{{ message.text?.content }} </template>
              <template v-else-if="message.metadata.message_type === MessageType.Join"> joined </template>
              <template v-else-if="message.metadata.message_type === MessageType.Part"> left </template>
              <template v-else> quit </template>
            </td>
          </tr>
        </table>
        <div id="scroll-anchor"></div>
      </div>
    </div>

    <div class="o-window-composer">
      <Composer @send="sendMessage" />
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
    height: 52px;
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
      padding-bottom: var(--space-s);
      overflow-y: auto;

      #scroll-anchor {
        overflow-anchor: auto;
        height: 1px;
      }

      .o-message-table {
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

          &.message-username {
            color: var(--color-text-lighter);
          }

          &.status {
            color: var(--color-text-light);
          }

          &:nth-child(1) {
            padding-left: var(--space-m);
          }

          &:nth-child(2) {
            width: 100%;
          }
        }
      }
    }
  }
}
</style>
