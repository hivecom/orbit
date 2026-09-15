<script setup lang="ts">
import type { Capabilities, Capability } from "core-wasm"
import { toJSON } from "../../../lib/helpers"
import { computed } from "vue"
import { Badge } from "@dolanske/vui"

const props = defineProps<{
  capabilities: Capabilities
}>()

const parsed = computed(() => toJSON<Record<string, Capability>>(props.capabilities))
</script>

<template>
  <dl>
    <template v-for="(value, key) in parsed">
      <dt class="ws-nowrap">{{ key }}</dt>
      <dd>
        <Badge size="s" :variant="value.enabled ? 'success' : 'danger'">
          {{ value.enabled }}
        </Badge>
      </dd>
    </template>
  </dl>
</template>
