<script setup lang="ts">
import { RouterView } from "vue-router"
import "./style/index.css"
import { DropdownItem, Sidebar } from "@dolanske/vui"
import { IconSettingsLinear, IconSidebarMinimalisticLinear } from "@iconify-prerendered/vue-solar"
import { useStorage } from "@vueuse/core"
// The main App entrypoint for orbit. It replaces the usual `App.vue` with an
// exportable component consumed by target applications (desktop/mobile/web).

// Treat this as a global layout. Navigation, header or other globally available
// components should live here.

const minimalSidebar = useStorage("orbit-sidebar-state", true)
</script>

<template>
  <div class="o-root vui-sidebar-layout">
    <Sidebar :mini="minimalSidebar">
      <DropdownItem @click="minimalSidebar = !minimalSidebar">
        <template #icon>
          <IconSidebarMinimalisticLinear />
        </template>
        Collapse sidebar
      </DropdownItem>

      <template #footer>
        <RouterLink to="/settings">
          <DropdownItem>
            <template #icon>
              <IconSettingsLinear />
            </template>

            Settings
          </DropdownItem>
        </RouterLink>
      </template>
    </Sidebar>
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
