<script setup lang="ts">
import { onMounted, useTemplateRef } from "vue"

interface Props {
  placeholder?: string
}

const { placeholder = "Write a message..." } = defineProps<Props>()

const emit = defineEmits<{
  send: [text: string]
}>()

const message = defineModel<string>({
  default: "",
})

function submit() {
  if (message.value) {
    emit("send", message.value)
    message.value = ""
  }
}

const input = useTemplateRef("inputRef")

onMounted(() => {
  requestAnimationFrame(() => {
    input.value?.focus()
  })
})
</script>

<template>
  <form @submit.prevent="submit" class="o-composer">
    <input type="text" v-model="message" :placeholder ref="inputRef" />
  </form>
</template>

<style scoped>
.o-composer {
  display: block;
  width: 100%;
  border-top: 1px solid var(--color-border);
  background-color: var(--color-bg-raised);
  border-radius: var(--border-radius-m);
  corner-shape: squircle;
  /* border-top-left-radius: unset;
  border-top-right-radius: unset; */

  input {
    border-radius: inherit;
    display: block;
    width: 100%;
    border: none;
    height: 52px;
    padding-inline: var(--space-m);
    padding-block: var(--space-xxxs);
    background-color: transparent;
    border: none;
    font-size: var(--font-size-m);
  }
}
</style>
