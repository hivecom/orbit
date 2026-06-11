<script setup lang="ts">
import "../style/index.css"

interface Props {
  environment: "web" | "mobile" | "native"
}

const props = defineProps<Props>()
</script>

<template>
  <div class="ob-root">
    <RouterView></RouterView>
  </div>
</template>
