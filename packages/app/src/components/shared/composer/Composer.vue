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
    emit("send", message.value.trim())
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
    <textarea v-model="message" :placeholder ref="inputRef" @keydown.enter.exact.prevent="submit" />
  </form>
</template>

<style scoped>
.o-composer {
  display: block;
  width: calc(100% - calc(var(--space-xs) * 2));
  background-color: var(--color-bg-medium);
  border-radius: var(--border-radius-m);
  corner-shape: squircle;
  margin: var(--space-xs);
  margin-top: 0;

  textarea,
  input {
    font-family: var(--font);
    border-radius: inherit;
    display: block;
    width: 100%;
    border: none;
    height: unset;
    min-height: 40px;
    field-sizing: content;
    resize: none;
    padding-inline: var(--space-m);
    padding-block: var(--space-s);
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
