<script setup lang="ts">
import { usePlatform } from "connector"

const platform = usePlatform()
</script>

<template>
  <div :data-test-target="platform.target" />
</template>
