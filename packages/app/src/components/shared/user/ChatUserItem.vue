<script setup lang="ts">
import { Badge, DropdownItem } from "@dolanske/vui"
import { getUserRole } from "../../../lib/format"
import { ChannelUser } from "core-wasm"
import { computed } from "vue"

const props = defineProps<{
  user: ChannelUser
}>()

const role = computed(() => getUserRole(props.user.role, true))
</script>

<template>
  <DropdownItem>
    {{ user.nickname }}
    <template #hint>
      <Badge circle size="s" :variant="role.badgeType as any" v-if="role">
        {{ role.label }}
      </Badge>
    </template>
  </DropdownItem>
</template>
