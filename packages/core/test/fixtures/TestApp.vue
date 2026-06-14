<script setup lang="ts">
import { usePlatform } from "platform"
import OrbitApp from "../../src/components/OrbitApp.vue"

const platform = usePlatform()
</script>

<template>
  <div :data-test-target="platform.target" />
</template>
