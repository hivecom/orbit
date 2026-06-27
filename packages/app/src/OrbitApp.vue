<script setup lang="ts">
import "../style/index.css"
// The main App entrypoint for orbit. It replaces the usual `App.vue` with an
// exportable component consumed by target applications (desktop/mobile/web).

// Since it is a router wrapper, we shouldn't really touch it. For global
// initialization, use the `setup.ts` file instead.
</script>

<template>
  <div class="ob-root">
    <RouterView />
  </div>
</template>
