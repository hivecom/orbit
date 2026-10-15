<script setup lang="ts">
import { computed, defineComponent } from "vue"
import { parseMessage } from "../../../lib/parse-message"

const props = defineProps<{
  content: string
}>()

const comp = defineComponent({
  props: { content: String },
  setup({ content }) {
    const result = computed(() => parseMessage(content))
    return () => result.value
  },
})
</script>

<template>
  <component :is="comp" :content="props.content"></component>
</template>
