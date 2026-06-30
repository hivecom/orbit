<script setup lang="ts">
import { Button } from "@dolanske/vui"
import type { WindowType } from "../../lib/windows"

// When button is clicked, the manager will open a new window in this location

const emit = defineEmits<{
  open: [type: WindowType]
}>()
</script>

<template>
  <div>
    <Button>Open chat</Button>
    <Button>Open voice</Button>
  </div>
</template>
