<script setup lang="ts">
import { type WindowAndLocation, type WindowChat } from "../../lib/windows"
import { useIrcStore } from "../../stores/irc"
import Composer from "../shared/composer/Composer.vue"
import { Accordion, Avatar, Button, DropdownItem, Flex, Grid } from "@dolanske/vui"
import { computed, nextTick, onMounted, ref, useTemplateRef, watch } from "vue"
import { useEventListener, useResizeObserver, useThrottleFn } from "@vueuse/core"
import { IRC_UNKNOWN_CHANNEL, IRC_UNKNOWN_SERVER } from "../../lib/constants.ts"
import { useIRCJoinChannel } from "../../composables/useIRCJoinChannel.ts"
import { useDateFormatter } from "../../lib/date.ts"
import { useConfigStore } from "../../stores/config.ts"
import { getServerInitials } from "../../lib/format.ts"
import { IconArrowDownLinear, IconUsersGroupRoundedLinear } from "@iconify-prerendered/vue-solar"
import { getUserColorStyle } from "../../lib/color.ts"
import ChatUserList from "../shared/user/ChatUserList.vue"

const props = defineProps<WindowAndLocation<WindowChat>>()
const irc = useIrcStore()
const config = useConfigStore()
const format = useDateFormatter()

// Get all messages per this channel. Setting to hide status message optionally filters them out
const messages = computed(() => {
  const data = irc.getChannelMessages(props.serverId, props.channelId)
  if (!config.options.appearance_chat_show_status_messages) {
    return data?.filter((msg) => msg.metadata.message_type === "Privmsg")
  }
  return data
})

// Channel related information
const channel = computed(() => irc.getServerChannel(props.serverId, props.channelId))

// Send a message & handle scroll
async function sendMessage(message: string) {
  if (!channel.value) return

  forceScroll = true
  irc.sendMessage(props.serverId, props.channelId, message)

  // Await DOM update in case the Composer height shrinks after clearing text
  await nextTick()
  if (scrollContainer.value) {
    scrollContainer.value.scrollTo({ top: scrollContainer.value.scrollHeight })
  }
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

// Scroll-related functionality
const scrollLoading = ref(false)
const scrollContainer = useTemplateRef("chatScrollContainer")
// How many pixels from the top of the chat will trigger load of more messages
const SCROLL_THRESHOLD = 200
const SCROLL_DOWN_THRESHOLD = 1000

// How many pixels form the bottom won't trigger scroll down
const BOTTOM_SCROLL_THRESHOLD = 50

let forceScroll = false
let isAtBottom = true
const showScrollDown = ref(false)

const throttledScrollCheck = useThrottleFn(
  async (event: Event) => {
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

    isAtBottom = Math.ceil(target.scrollTop + target.clientHeight) >= target.scrollHeight - BOTTOM_SCROLL_THRESHOLD

    if (target.scrollTop + target.clientHeight < target.scrollHeight - SCROLL_DOWN_THRESHOLD) {
      showScrollDown.value = true
    } else {
      showScrollDown.value = false
    }
  },
  100,
  true,
)

useEventListener(scrollContainer, "scroll", throttledScrollCheck)

// When the container shrinks (e.g. Composer textarea grows), keep the chat pinned to the bottom
useResizeObserver(scrollContainer, () => {
  const el = scrollContainer.value
  if (el && isAtBottom) {
    el.scrollTop = el.scrollHeight
  }
})

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

// Scroll down
function scrollDown() {
  const el = scrollContainer.value
  if (el) {
    el.scrollTo({ top: el.scrollHeight, behavior: config.options.appearance_chat_smooth_scroll ? "smooth" : "auto" })
  }
}

// Check for channel ID and scroll to the bottom when joining / rendering it for the first time
watch(
  () => props.channelId,
  async () => {
    await nextTick()
    scrollDown()
  },
  { immediate: true },
)

// Watch for messages and scroll if we should or not
watch(
  messages,
  async () => {
    const el = scrollContainer.value
    if (!el) return
    // Check size of chat BEFORE dom updates (flush: pre)
    const wasAtBottom = Math.ceil(el.scrollTop + el.clientHeight) >= el.scrollHeight - BOTTOM_SCROLL_THRESHOLD
    await nextTick()
    // Now the DOM updated and we scroll to the new pos
    if (wasAtBottom || forceScroll) {
      el.scrollTo({ top: el.scrollHeight, behavior: config.options.appearance_chat_smooth_scroll ? "smooth" : "auto" })
      forceScroll = false
    }
  },
  { flush: "pre" },
)

// Compute message date splitters as ids. When rendering messages each message
// compares its id with this set and if it matches, it inserts a splitter row
// before itself
const dateIds = computed(() => {
  const ids = new Set()
  if (!messages.value) return ids
  let prev = ""
  for (const m of messages.value) {
    const key = new Date(m.metadata.server_time).toDateString()
    if (key !== prev) {
      ids.add(m.metadata.msgid)
      prev = key
    }
  }
  return ids
})

// UI settings
const showUserList = ref(false)
</script>

<template>
  <div class="o-window-chat" :class="{ 'show-users': showUserList }">
    <div class="o-window-meta" v-if="props.channelId !== IRC_UNKNOWN_CHANNEL">
      <p>{{ props.channelId }}</p>
      <p class="text-xs text-color-lighter meta-topic">{{ channel?.metadata.topic }}</p>

      <div class="flex-1"></div>

      <Button @click="showUserList = !showUserList" plain>
        <template #start>
          <IconUsersGroupRoundedLinear />
        </template>
        {{ channel?.users.length }}
      </Button>
    </div>
    <ChatUserList v-if="showUserList && channel" :users="channel.users" />
    <div class="o-window-chat-wrap">
      <div class="o-channel-list" v-if="props.channelId === IRC_UNKNOWN_CHANNEL || props.serverId === IRC_UNKNOWN_SERVER">
        <!-- 
    Render all servers
    TODO: still needs to be tested
      1. if server id is -1 we open first accordion
      2. if server is id real, we open accordion of that server -->
        <Flex column x-center y-center class="h-100">
          <div class="container-s">
            <Accordion card ref="accordionRef" v-for="server in irc.serversWithChannels" :data-server="server.id" :key="server.id">
              <template #header>
                <Flex y-center>
                  <Avatar size="m">
                    {{ getServerInitials(server.metadata) }}
                  </Avatar>
                  <p>{{ server.metadata.name ?? server.metadata.address }}</p>
                </Flex>
              </template>
              <Grid :columns="4">
                <DropdownItem :disabled="loadingChannel" v-for="channel in server.groupedChannels.joined" :key="channel.metadata.name" @click="join(server.id, channel.metadata.name, { forceLocation: props.location })">
                  {{ channel.metadata.name }}
                </DropdownItem>
                <DropdownItem class="lighter" :disabled="loadingChannel" v-for="channel in server.groupedChannels.available" :key="channel.name" @click="join(server.id, channel.name, { forceLocation: props.location })">
                  {{ channel.name }}
                </DropdownItem>
              </Grid>
            </Accordion>
          </div>
        </Flex>
      </div>
      <div class="o-table-wrap" v-else :style="chatPositionStyle">
        <Button square class="o-btn-scroll-down" @click="scrollDown()" :class="{ active: showScrollDown }" aria-label="Scroll to latest message">
          <IconArrowDownLinear />
        </Button>

        <div class="o-table-scroll-container" ref="chatScrollContainer">
          <table class="o-msg-table">
            <template v-for="message in messages" :key="message.metadata.msgid">
              <tr v-if="dateIds.has(message.metadata.msgid)" class="msg-date-splitter">
                <td colspan="4">
                  <svg width="20" height="44" viewBox="0 0 20 44" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M0 23C10 23 20 23.5 20 44V22V0C20 21 10 21 0 21V23Z" fill="currentColor" />
                  </svg>
                  <span>{{ format.simple(message.metadata.server_time) }}</span>
                  <svg width="20" height="44" viewBox="0 0 20 44" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M0 23C10 23 20 23.5 20 44V22V0C20 21 10 21 0 21V23Z" fill="currentColor" />
                  </svg>
                </td>
              </tr>
              <tr>
                <td class="msg-timestamp" v-if="config.options.appearance_chat_timestamps_enabled">{{ format.chatTimestamp(message.metadata.server_time) }}</td>
                <td class="msg-username user-color" :style="getUserColorStyle(message.metadata.user)">{{ message.metadata.user }}</td>
                <td class="msg-content" :class="{ status: message.metadata.message_type !== 'Privmsg' }">
                  <template v-if="message.metadata.message_type === 'Privmsg'">{{ message.text?.content }} </template>
                  <template v-else-if="message.metadata.message_type === 'Join'"> joined </template>
                  <template v-else-if="message.metadata.message_type === 'Part'"> left </template>
                  <template v-else> quit </template>
                </td>
              </tr>
            </template>
          </table>
          <a id="scroll-anchor"></a>
        </div>
      </div>

      <div class="o-window-composer" v-if="props.channelId !== IRC_UNKNOWN_CHANNEL" :style="composerPositionStyle">
        <Composer @send="sendMessage" :placeholder="`Message ${props.channelId}`" />
      </div>
    </div>
  </div>
</template>
<style scoped>
.o-window-chat {
  display: grid;
  height: 100%;
  width: 100%;
  grid-template-rows: 44px 1fr;
  grid-template-columns: 1fr 224px;
  grid-template-areas:
    "meta meta"
    "chat chat";

  &.show-users {
    grid-template-areas:
      "meta meta"
      "chat users";
  }

  .o-window-meta {
    display: flex;
    width: 100%;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-m);
    padding-inline: var(--space-s);
    border-bottom: 1px solid var(--color-border);
    height: 44px;
    background-color: var(--color-bg-lowered);
    z-index: 5;
    border-top-left-radius: var(--border-radius-m);
    border-top-right-radius: var(--border-radius-m);
    grid-area: meta;

    .meta-topic {
      text-overflow: ellipsis;
      white-space: nowrap;
      overflow: hidden;
    }
  }

  .o-window-users {
    height: 100%;
    border-radius: none;
    background-color: var(--color-bg-lowered);
    border-bottom-right-radius: var(--border-radius-m);
    padding: var(--space-s);
    position: relative;
    z-index: 2;
    border-left: 1px solid var(--color-border-weak);
  }

  .o-window-chat-wrap {
    height: 100%;
    width: 100%;
    display: flex;
    flex-direction: column;
    grid-area: chat;

    .o-window-composer {
      position: sticky;
      bottom: 0;
    }

    .o-channel-list {
      flex: 1;
    }

    .o-btn-scroll-down {
      position: absolute;
      bottom: 0;
      left: 50%;
      transform: translate(-50%, 0);
      z-index: -1;
      opacity: 0;
      transition: all var(--transition);
      visibility: hidden;

      &.active {
        opacity: 1;
        z-index: 10;
        visibility: visible;
        transform: translate(-50%, -16px);
      }
    }

    .o-table-wrap {
      flex: 1;
      position: relative;

      .o-table-scroll-container {
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        max-height: 100%;
        padding-bottom: var(--space-xs);
        overflow-y: auto;

        #scroll-anchor {
          display: block;
          overflow-anchor: auto;
          height: 1px;
        }

        .o-msg-table {
          table-layout: auto;
          overflow-anchor: none;
          padding-top: var(--space-m);

          .msg-date-splitter {
            td {
              text-align: center;
              color: var(--color-text-lightest);
              position: relative;

              svg {
                position: absolute;
                left: 0;
                top: 50%;
                transform: translateY(-50%) scaleX(-1);
                color: var(--color-bg);

                &:last-of-type {
                  left: unset;
                  right: 0;
                  transform: translateY(-50%);
                }
              }

              &:before {
                content: "";
                position: absolute;
                height: 2px;
                background: var(--color-bg);
                left: 8px;
                right: 8px;
                top: 50%;
                transform: translateY(-50%);
              }

              span {
                background-color: var(--color-bg-lowered);
                font-size: var(--font-size-xs);
                z-index: 2;
                position: relative;
                padding-inline: var(--space-m);
              }
            }
          }

          tr:not(.msg-date-splitter):hover td {
            background-color: var(--color-bg);
          }

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
              /* --user-color: var(--color-text-light);
              color: var(--user-color); */
              padding-right: var(--space-xs);
              padding-left: var(--space-xs);
            }

            &.msg-content {
              width: 100%;
            }

            &.status {
              color: var(--color-text-lighter);
              font-style: italic;
            }

            &:nth-child(1) {
              padding-left: var(--space-m);
            }
          }
        }
      }
    }
  }
}
</style>
