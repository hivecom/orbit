<script setup lang="ts">
import type { ChannelUser } from "core/types"
import ChatUserItem from "./ChatUserItem.vue"
import { computed } from "vue"

const props = defineProps<{
  users: ChannelUser[]
}>()

const sortedUsers = computed(() => props.users.sort((a, b) => (a.role > b.role ? 1 : -1)))
</script>

<template>
  <div class="o-window-users">
    <ChatUserItem v-for="user in sortedUsers" :key="user.nickname" :user="user" />
  </div>
</template>
