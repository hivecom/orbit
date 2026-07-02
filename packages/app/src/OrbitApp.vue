<script setup lang="ts">
import { RouterView } from "vue-router"
import "./style/index.css"
import GlobalSidebar from "./components/navigation/GlobalSidebar.vue"

// The main App entrypoint for orbit. It replaces the usual `App.vue` with an
// exportable component consumed by target applications (desktop/mobile/web).

// Treat this as a global layout. Navigation, header or other globally available
// components should live here.
</script>

<template>
  <div class="o-root vui-sidebar-layout">
    <GlobalSidebar />
    <main class="h-100">
      <RouterView />
    </main>
  </div>
</template>

<style>
.vui-sidebar {
  &.mini {
    --vui-sidebar-width: 60px !important;
  }
}
</style>
