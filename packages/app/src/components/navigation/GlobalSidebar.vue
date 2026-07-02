<script setup lang="ts">
import { Avatar, Divider, Flex, DropdownItem, Indicator, Sidebar } from "@dolanske/vui"
import { IconMagniferLinear, IconSettingsLinear, IconSidebarMinimalisticLinear } from "@iconify-prerendered/vue-solar"
import { useStorage } from "@vueuse/core"

const minimalSidebar = useStorage("orbit-sidebar-state", true)
</script>

<template>
  <Sidebar :mini="minimalSidebar">
    <DropdownItem @click="minimalSidebar = !minimalSidebar">
      <template #icon>
        <IconSidebarMinimalisticLinear />
      </template>
      Collapse sidebar
    </DropdownItem>

    <Divider class="my-m" />

    <Flex column gap="xs">
      <DropdownItem>
        <template #icon>
          <IconMagniferLinear />
        </template>
        Search
      </DropdownItem>

      <DropdownItem>
        <template #icon>
          <Avatar :size="minimalSidebar ? 'm' : 's'">
            H
            <template #overlay>
              <Indicator variant="alert" size="s" position="top-right" />
            </template>
          </Avatar>
        </template>
        Hivecom
      </DropdownItem>
    </Flex>

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
</template>
