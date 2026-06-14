<script setup lang="ts">
import { usePlatform } from "platform"

const platform = usePlatform()
</script>

<template>
  <div :data-test-target="platform.target" />
</template>
