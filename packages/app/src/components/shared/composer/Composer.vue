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
  background-color: var(--color-bg-medium);
  border-radius: var(--border-radius-m);
  border-top-left-radius: 0;
  border-top-right-radius: 0;
  corner-shape: squircle;

  input {
    border-radius: inherit;
    display: block;
    width: 100%;
    border: none;
    height: 44px;
    padding-inline: var(--space-m);
    padding-block: var(--space-xxxs);
    background-color: transparent;
    border: none;
    font-size: var(--font-size-m);

    &:focus-within {
      outline: none;
      background-color: var(--color-bg-raised);
    }
  }
}
</style>
