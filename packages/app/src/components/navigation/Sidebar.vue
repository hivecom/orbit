<script setup lang="ts">
import { Avatar, Divider, Flex, DropdownItem, Sidebar, Card, Button, PopoutHover, Input, searchString } from "@dolanske/vui"
import { IconAddCircleLinear, IconMagniferLinear, IconSettingsLinear, IconSidebarMinimalisticLinear } from "@iconify-prerendered/vue-solar"
import { useStorage } from "@vueuse/core"
import { useIrcStore } from "../../stores/irc"
import ListCapabilities from "../shared/server/ListCapabilities.vue"
import { truncate } from "../../lib/format.ts"
import { computed, ref } from "vue"
import { useConfigStore } from "../../stores/config.ts"

const irc = useIrcStore()
const config = useConfigStore()

config.onShortcut("global:navigation-toggle", () => {
  mini.value = !mini.value
})

const mini = useStorage("orbit-sidebar-state", true)

// Search through servers
// TODO: mini-sidebar search
const search = ref("")
const serversRaw = computed(() => Array.from(irc.serverData.values()))
const filteredServers = computed(() => serversRaw.value.filter((server) => searchString([server.metadata.name, server.metadata.address], search.value)))
</script>

<template>
  <Sidebar :mini="mini">
    <Flex column gap="xxs">
      <!-- Minified -->
      <template v-if="mini">
        <DropdownItem @click="mini = !mini" aria-label="Expand sidebar">
          <template #icon>
            <IconSidebarMinimalisticLinear />
          </template>
        </DropdownItem>
        <DropdownItem aria-label="Search servers">
          <template #icon>
            <IconMagniferLinear />
          </template>
        </DropdownItem>
      </template>

      <!-- Expanded -->
      <template v-else>
        <Flex gap="xs" class="p-xxxs">
          <Button square @click="mini = !mini" aria-label="Collapse sidebar">
            <IconSidebarMinimalisticLinear />
          </Button>
          <Input aria-label="Search servers" v-model="search" placeholder="Search" style="--vui-input-width: auto" />
        </Flex>
      </template>
    </Flex>

    <div style="height: 1px" />
    <Divider type="dashed" class="my-m" />

    <Flex column gap="xxs">
      <template v-for="server in filteredServers" :key="server.metadata.name">
        <PopoutHover :enter-delay="1000" class="o-server-info">
          <template #trigger>
            <DropdownItem class="o-sidebar-server-item">
              <template #icon>
                <Avatar :size="mini ? 'm' : 's'">
                  <!-- FIXME: clean the fallback up and consider irc.<network>.<tld> cases -->
                  {{ (server.metadata.name?.charAt(0) ?? server.metadata.address.startsWith("wss://")) ? server.metadata.address.charAt(6).toUpperCase() : server.metadata.address.charAt(0) }}
                  <!-- <template #overlay>
              <Indicator variant="alert" size="s" position="top-right" />
            </template> -->
                </Avatar>
              </template>
              {{ truncate(server.metadata.name ?? server.metadata.address, 20, "..") }}
            </DropdownItem>
          </template>
          <ListCapabilities :capabilities="server.capabilities" />
        </PopoutHover>
      </template>

      <RouterLink to="/" class="w-100">
        <DropdownItem>
          <template #icon>
            <IconAddCircleLinear />
          </template>
          Connect
        </DropdownItem>
      </RouterLink>
    </Flex>

    <template #footer>
      <!-- Minified -->
      <DropdownItem v-if="mini" x-center expand>
        <template #icon>
          <Avatar url="https://github.com/dolanske.png" size="m"></Avatar>
        </template>
      </DropdownItem>

      <!-- Expanded -->
      <Card class="o-sidebar-user" v-else>
        <Flex y-center gap="xs" expand>
          <Avatar url="https://github.com/dolanske.png"></Avatar>
          <strong class="flex-1">dolanske</strong>
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

.o-sidebar-server-item {
  overflow: hidden;
}

.o-server-info {
  width: 256px;
}
</style>
