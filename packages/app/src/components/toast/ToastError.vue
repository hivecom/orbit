<script setup lang="ts">
import { Button, Card, removeToast } from "@dolanske/vui"
import type { OrbitError } from "core/types"

const props = defineProps<{
  title?: string
  data: {
    title?: string
    error: OrbitError
  }
  toastId: number
}>()
</script>

<template>
  <Card header-separator>
    <template #header>
      <strong class="text-color-red">{{ props.data.title ?? "Error" }}</strong>
    </template>
    <template #header-end>
      <Button size="s" @click="removeToast(props.toastId)">Dismiss</Button>
    </template>
    <pre>{{ props.data.error }}</pre>
  </Card>
</template>
