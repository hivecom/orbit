<script setup lang="ts">
import { Badge, DropdownItem } from "@dolanske/vui"
import { getUserRole } from "../../../lib/format"
import type { ChannelUser } from "core-wasm"
import { computed } from "vue"
import { getUserColorStyle } from "../../../lib/color"

const props = defineProps<{
  user: ChannelUser
}>()

const role = computed(() => getUserRole(props.user.role, true))
</script>

<template>
  <DropdownItem>
    <span class="user-color" :style="getUserColorStyle(props.user.nickname)">
      {{ user.nickname }}
    </span>
    <template #hint>
      <Badge circle size="s" :variant="role.badgeType as any" v-if="role">
        {{ role.label }}
      </Badge>
    </template>
  </DropdownItem>
</template>
