<script setup lang="ts">
import { Avatar, Divider, Flex, DropdownItem, Indicator, Sidebar, Card, Button } from "@dolanske/vui"
import { IconMagniferLinear, IconSettingsLinear, IconSidebarMinimalisticLinear } from "@iconify-prerendered/vue-solar"
import { useStorage } from "@vueuse/core"

const miniBar = useStorage("orbit-sidebar-state", true)
</script>

<template>
  <Sidebar :mini="miniBar">
    <Flex column gap="xxs">
      <DropdownItem @click="miniBar = !miniBar">
        <template #icon>
          <IconSidebarMinimalisticLinear />
        </template>
        Collapse sidebar
      </DropdownItem>
    </Flex>

    <Divider type="dashed" class="my-m" />

    <Flex column gap="xxs">
      <DropdownItem>
        <template #icon>
          <IconMagniferLinear />
        </template>
        Search
      </DropdownItem>

      <DropdownItem>
        <template #icon>
          <Avatar :size="miniBar ? 'm' : 's'">
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
      <!-- TODO: minified sidebar is _just_ the avatar -->
      <Card class="o-sidebar-user">
        <Flex y-center gap="xs" expand>
          <Avatar url="https://github.com/dolanske.png"></Avatar>
          <Flex column gap="s" class="flex-1">
            <strong v-show="!miniBar">dolanske</strong>
          </Flex>
          <RouterLink to="/settings">
            <Button square plain>
              <IconSettingsLinear />
            </Button>
          </RouterLink>
        </Flex>
      </Card>
    </template>
  </Sidebar>
</template>

<style>
.o-sidebar-user {
  --vui-card-padding-block: var(--space-s);
  --vui-card-padding-inline: var(--space-s);
}
</style>
