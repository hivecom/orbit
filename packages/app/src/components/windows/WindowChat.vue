<script setup lang="ts">
import { MessageType } from "core-wasm"
import { type WindowAndLocation, type WindowChat } from "../../lib/windows"
import { useIrcStore } from "../../stores/irc"
import Composer from "../shared/composer/Composer.vue"
import { DropdownItem, Flex, Grid } from "@dolanske/vui"
import { computed, nextTick, ref, useTemplateRef } from "vue"
import { useEventListener, useThrottleFn } from "@vueuse/core"
import { IRC_UNKNOWN_CHANNEL } from "../../lib/constants.ts"
import { useIRCJoinChannel } from "../../composables/useIRCJoinChannel.ts"
import { useDateFormatter } from "../../lib/date.ts"

const props = defineProps<WindowAndLocation<WindowChat>>()
const irc = useIrcStore()

const format = useDateFormatter()

const messages = computed(() => irc.getChannelMessages(props.serverId, props.channelId))
const state = computed(() => irc.getServerState(props.serverId))
const channels = computed(() => irc.getServerChannels(props.serverId))
const channel = computed(() => irc.getServerChannel(props.serverId, props.channelId))

function sendMessage(message: string) {
  if (!channel.value) return
  channel.value.handler.send_message(message)
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
const { join, loading: loadingChannel } = useIRCJoinChannel()
</script>

<template>
  <div class="o-window-chat" v-if="state">
    <div class="o-window-meta" v-if="props.channelId !== IRC_UNKNOWN_CHANNEL">
      <p>{{ props.channelId }}</p>
    </div>
    <div class="o-channel-list" v-if="props.channelId === IRC_UNKNOWN_CHANNEL">
      <Flex column x-center y-center class="h-100">
        <div>
          <Grid :columns="4">
            <DropdownItem :disabled="loadingChannel" v-for="channel in channels?.joined" :key="channel.data.metadata.name" @click="join(props.serverId, channel.data.metadata.name, props.location)">
              {{ channel.data.metadata.name }}
            </DropdownItem>
            <DropdownItem class="lighter" :disabled="loadingChannel" v-for="channel in channels?.available" :key="channel.name" @click="join(props.serverId, channel.name, props.location)">
              {{ channel.name }}
            </DropdownItem>
          </Grid>
        </div>
      </Flex>
    </div>
    <div class="o-table-wrap" v-else>
      <div class="o-table-scroll-container" ref="chatScrollContainer">
        <table class="o-msg-table">
          <tr v-for="message in messages" :key="message.metadata.msgid">
            <td class="msg-timestamp">{{ format.chatTimestamp(message.metadata.server_time) }}</td>
            <td class="msg-username">{{ message.metadata.user }}</td>
            <td class="msg-content" :class="{ status: message.metadata.message_type !== MessageType.Privmsg }">
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

    <div class="o-window-composer" v-if="props.channelId !== IRC_UNKNOWN_CHANNEL">
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
