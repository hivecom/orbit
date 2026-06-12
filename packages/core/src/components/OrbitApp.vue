<script setup lang="ts">
import { usePlatform } from "../platform/index.ts"
import "../style/index.css"

// Environment is derived from the injected platform adapter - the single
// source of truth - rather than a prop. The adapter is wired in at app boot
// (see each entrypoint's main.ts).
const platform = usePlatform()
</script>

<template>
  <div class="ob-root" :data-target="platform.target">
    <RouterView />
  </div>
</template>
