<script setup lang="ts">
import { MessageType } from "core-wasm"
import type { WindowChat } from "../../lib/windows"
import { useIrcStore } from "../../stores/irc"
import Composer from "../shared/composer/Composer.vue"
import { Flex, PopoutHover } from "@dolanske/vui"
import { IconInfoCircleLinear } from "@iconify-prerendered/vue-solar"

// TODO: Figure out connecting to specific channels and showing the one that's open (and replacing url state)

const props = defineProps<WindowChat>()

const irc = useIrcStore()
const messages = irc.getChannelMessages(props.serverId, "#orbit/testing")

const state = irc.getServerState(props.serverId)

function sendMessage(message: string) {
  console.log(message)
  irc.serverChannel?.send_message(message)
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
    <!-- <pre>
      {{ props }}
    </pre>
    <pre>
      {{ state }}
    </pre>
    <br /> -->
    <!-- <button @click="irc.requestScrollback(props.serverId, '#orbit/testing')">Get more</button> -->
    <div class="o-table-wrap-outer">
      <div class="o-table-wrap-inner">
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

  .o-table-wrap-outer {
    flex: 1;
    padding-bottom: 52px;
    position: relative;

    .o-table-wrap-inner {
      overflow-anchor: none;
      position: absolute;
      inset: 0;
      display: flex;
      flex-direction: column;
      justify-content: flex-end;
      overflow-y: auto;
      padding-bottom: var(--space-s);

      #scroll-anchor {
        overflow-anchor: auto;
        height: 1px;
      }

      .o-message-table {
        table-layout: auto;

        /* tr:last-child td {
          border-bottom: none;
        } */

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
