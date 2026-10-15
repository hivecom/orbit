<script setup lang="ts">
import { computed } from "vue"
import { openMsgModal } from "../../../lib/parse-message"
import { CHAT_URL_TRUSTED_DOMAINS_KEY } from "../../../lib/constants"

const props = defineProps<{
  href: string
}>()

const isTrusted = computed(() => {
  const list = localStorage.getItem(CHAT_URL_TRUSTED_DOMAINS_KEY)
  if (!list) return false
  const parsed = JSON.parse(list) as string[]
  const url = new URL(props.href)
  return parsed.includes(url.hostname)
})
</script>

<template>
  <a v-if="isTrusted" class="o-chat-a" :href="props.href" noreferrer noopener target="_blank">
    {{ props.href }}
  </a>
  <button v-else class="o-chat-a" @click="openMsgModal('link', props.href)">{{ props.href }}</button>
</template>
