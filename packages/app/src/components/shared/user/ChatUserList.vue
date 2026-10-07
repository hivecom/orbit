<script setup lang="ts">
import { ChannelUser } from "core-wasm"
import ChannelUserItem from "./ChatUserItem.vue"
import { computed } from "vue"

const props = defineProps<{
  users: ChannelUser[]
}>()

const sortedUsers = computed(() => props.users.sort((a, b) => (a.role > b.role ? 1 : -1)))
</script>

<template>
  <div class="o-window-users">
    <Flex column gap="xs">
      <ChannelUserItem v-for="user in sortedUsers" :key="user.nickname" :user="user" />
    </Flex>
  </div>
</template>
