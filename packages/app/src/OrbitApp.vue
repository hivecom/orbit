<script setup lang="ts">
import { RouterView } from "vue-router"
import "./style/index.css"
import Sidebar from "./components/navigation/Sidebar.vue"
import { Flex, Spinner } from "@dolanske/vui"
import { useAppStateStore } from "./stores/app-state.ts"
import { usePlatform } from "platform"
import { onMounted } from "vue"

// The main App entrypoint for orbit. It replaces the usual `App.vue` with an
// exportable component consumed by target applications (desktop/mobile/web).

// Treat this as a global layout. Navigation, header or other globally available
// components should live here.
const appState = useAppStateStore()
const platform = usePlatform()

onMounted(async () => {
  if (platform.tray) {
    platform.tray.setTitle("Hello from title")
    platform.tray.setBadgeCount(2)
  }
})
</script>

<template>
  <Flex class="o-root" v-if="appState.globalError" y-center x-center>
    <h3 class="text-color-red">Global error</h3>
    <p>{{ appState.globalError }}</p>
  </Flex>

  <div class="o-root vui-sidebar-layout" v-else-if="!appState.initialized">
    <Sidebar />
    <main class="h-100">
      <RouterView />
    </main>
  </div>

  <Flex x-center y-center v-else class="o-fullscreen-loading" column gap="s">
    <h3>Initializing Orbit</h3>
    <p class="text-color-light mb-l">Orbit was conceptualized in 2022 and since then, 2 failed versions have been developed. This is the third and final one.</p>
    <Spinner />
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

  p {
    max-width: 512px;
    text-align: center;
    line-height: var(--line-height-loose);
  }
}
</style>
