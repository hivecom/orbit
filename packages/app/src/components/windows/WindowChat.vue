<script setup lang="ts">
import { MessageType } from "core-wasm"
import type { WindowChat } from "../../lib/windows"
import { useIrcStore } from "../../stores/irc"
import { ref } from "vue"

// TODO: if no chat is active, list available chats for a server. We know that
// because channelId will be "__unspecified"

const props = defineProps<WindowChat>()

const editmsg = ref("")

const irc = useIrcStore()
const messages = irc.getChannelMessages(props.serverId, "#orbit/testing")
// const channel = irc.get

async function sendMessage() {
  if (editmsg.value === "") {
    return
  }
  const out = editmsg.value
  editmsg.value = ""
  await irc.serverChannel?.send_message(out)
}
</script>

<template>
  <div>
    <pre>
      {{ props }}
    </pre>
    <button @click="irc.requestScrollback(props.serverId, '#orbit/testing')">Get more</button>
    <div class="messages">
      <div v-for="message in messages" :key="message.metadata.msgid">
        <pre class="msg" v-if="message.metadata.message_type === MessageType.Privmsg">{{ message.metadata.user }}: {{ message.text?.content }}</pre>
        <pre class="join" v-if="message.metadata.message_type === MessageType.Join">{{ message.metadata.user }} joined</pre>
        <pre class="part" v-if="message.metadata.message_type === MessageType.Part">{{ message.metadata.user }} parted</pre>
        <pre class="quit" v-if="message.metadata.message_type === MessageType.Quit">{{ message.metadata.user }} quit</pre>
      </div>
    </div>
    <form @submit.prevent="sendMessage">
      <input v-model="editmsg" type="text" />
    </form>
  </div>
</template>
<style scoped>
button {
  padding: 10px;
  background-color: gray;
}
.messages {
  max-height: 500px;
  width: 800px;
  overflow-y: scroll;

  pre {
    padding: 2px;
  }

  .join {
    color: green;
  }
  .part,
  .quit {
    color: red;
  }
}
</style>
