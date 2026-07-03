<script setup lang="ts">
import { RouterView } from "vue-router"
import "./style/index.css"
import NavigationSidebar from "./components/navigation/NavigationSidebar.vue"
import { Flex, Spinner } from "@dolanske/vui"
import { useAppStateStore } from "./stores/app-state.ts"

// The main App entrypoint for orbit. It replaces the usual `App.vue` with an
// exportable component consumed by target applications (desktop/mobile/web).

// Treat this as a global layout. Navigation, header or other globally available
// components should live here.
const appState = useAppStateStore()
</script>

<template>
  <Flex class="o-root" v-if="appState.globalError" y-center x-center>
    <h5 class="text-color-red">Global error</h5>
    <p>{{ appState.globalError }}</p>
  </Flex>

  <div class="o-root vui-sidebar-layout" v-if="appState.initialized">
    <NavigationSidebar />
    <main class="h-100">
      <RouterView />
    </main>
  </div>

  <Flex x-center y-center v-else class="o-fullscreen-loading" column gap="s">
    <Spinner />
    <h5>Initializing Orbit</h5>
    <p>Did you know Orbit was conceptualized in 2022 and since 2 failed versions have been developed?</p>
  </Flex>
</template>

<style>
.vui-sidebar {
  &.mini {
    --vui-sidebar-width: 60px !important;
  }
}

.o-fullscreen-loading {
  position: fixed;
  inset: 0;
}
</style>
