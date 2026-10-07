<script setup lang="ts">
import type { Capabilities } from "core-wasm"
import { Badge } from "@dolanske/vui"

const props = defineProps<{ capabilities: Capabilities }>()
</script>

<template>
  <dl>
    <template v-for="(value, key) in props.capabilities">
      <dt class="ws-nowrap">{{ key }}</dt>
      <dd>
        <Badge size="s" :variant="value.enabled ? 'success' : 'danger'">
          {{ value.enabled }}
        </Badge>
      </dd>
    </template>
  </dl>
</template>
