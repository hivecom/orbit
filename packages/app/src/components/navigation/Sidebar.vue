<script setup lang="ts">
import { Avatar, Divider, Flex, DropdownItem, Sidebar, Card, Button, Tooltip } from "@dolanske/vui"
import { IconSettingsLinear, IconSidebarMinimalisticLinear } from "@iconify-prerendered/vue-solar"
import { useStorage } from "@vueuse/core"
import { useIrcStore } from "../../stores/irc"

const irc = useIrcStore()

const mini = useStorage("orbit-sidebar-state", true)
</script>

<template>
  <Sidebar :mini="mini">
    <Flex column gap="xxs">
      <DropdownItem @click="mini = !mini">
        <template #icon>
          <IconSidebarMinimalisticLinear />
        </template>
        Collapse sidebar
      </DropdownItem>
    </Flex>

    <Divider type="dashed" class="my-m" />

    <Flex column gap="xxs">
      <!-- <DropdownItem>
        <template #icon>
          <IconMagniferLinear />
        </template>
        Search
      </DropdownItem> -->

      <DropdownItem v-for="server in irc.serverData.values()" :key="server.metadata.name">
        <template #icon>
          <Avatar :size="mini ? 'm' : 's'">
            <!-- FIXME: clean the fallback up and consider irc.<network>.<tld> cases -->
            {{ (server.metadata.name?.charAt(0) ?? server.metadata.address.startsWith("wss://")) ? server.metadata.address.charAt(6) : server.metadata.address.charAt(0) }}
            <!-- <template #overlay>
              <Indicator variant="alert" size="s" position="top-right" />
            </template> -->
          </Avatar>
        </template>
        {{ server.metadata.name }}
      </DropdownItem>
    </Flex>

    <template #footer>
      <!-- TODO: minified sidebar is _just_ the avatar -->
      <Flex v-if="mini" x-center expand>
        <Avatar url="https://github.com/dolanske.png"></Avatar>
      </Flex>

      <Card class="o-sidebar-user" v-else>
        <Flex y-center gap="xs" expand>
          <Avatar url="https://github.com/dolanske.png"></Avatar>
          <Flex column gap="s" class="flex-1">
            <Tooltip>
              <strong>dolanske</strong>
            </Tooltip>
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

  strong {
    display: block;
    width: 100%;
    white-space: nowrap;
    text-overflow: ellipsis;
    overflow: hidden;
  }
}
</style>
