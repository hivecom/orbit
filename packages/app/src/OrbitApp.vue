<script setup lang="ts">
import { RouterView, useRoute } from "vue-router"
import "./style/index.css"
import Sidebar from "./components/navigation/Sidebar.vue"
import { Flex, Spinner } from "@dolanske/vui"
import { useAppStateStore } from "./stores/app-state.ts"

// The main App entrypoint for orbit. It replaces the usual `App.vue` with an
// exportable component consumed by target applications (desktop/mobile/web).

// Treat this as a global layout. Navigation, header or other globally available
// components should live here.
const appState = useAppStateStore()
const route = useRoute()
</script>

<template>
  <Flex class="o-root" v-if="appState.globalError" y-center x-center>
    <h3 class="text-color-red">Global error</h3>
    <p>{{ appState.globalError }}</p>
  </Flex>

  <div class="o-root vui-sidebar-layout" v-else-if="!appState.initialized">
    <Sidebar />
    <main class="o-main">
      <div class="o-wrap" :class="{ bordered: !route.path.startsWith('/wm') }">
        <RouterView />
      </div>
    </main>
  </div>

  <Flex x-center y-center v-else class="o-fullscreen-loading" column gap="s">
    <h3>Initializing Orbit</h3>
    <p class="text-color-light mb-l">Spinning up the core</p>
    <Spinner />
  </Flex>
</template>

<style>
.o-main {
  height: 100%;
  width: 100%;

  .o-wrap {
    height: 100%;
    width: 100%;

    &.bordered {
      background-color: var(--color-bg-lowered);
      --wrap-padding: var(--space-s);
      border: 1px solid var(--color-border-weak);
      border-radius: var(--border-radius-m);
      width: calc(100% - calc(var(--wrap-padding) * 2));
      height: calc(100% - calc(var(--wrap-padding) * 2));
      padding: var(--wrap-padding);
      margin: var(--wrap-padding);
      corner-shape: squircle;
    }
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
