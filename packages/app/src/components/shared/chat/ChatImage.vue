<script setup lang="ts">
const props = defineProps<{
  src: string
}>()
</script>

<template>
  <img :src="props.src" class="o-chat-img" alt="Chat Image" />
</template>
