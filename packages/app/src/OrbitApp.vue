<script setup lang="ts">
import { RouterView } from "vue-router"
import "./style/index.css"
// The main App entrypoint for orbit. It replaces the usual `App.vue` with an
// exportable component consumed by target applications (desktop/mobile/web).

// Treat this as a global layout. Navigation, header or other globally available
// components should live here.
</script>

<template>
  <div class="o-root">
    <RouterView />
  </div>
</template>
