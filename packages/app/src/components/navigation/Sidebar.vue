<script setup lang="ts">
import { Avatar, Divider, Flex, DropdownItem, Sidebar, Card, Button, PopoutHover, Input, searchString } from "@dolanske/vui"
import { IconAddCircleLinear, IconMagniferLinear, IconSettingsLinear, IconSidebarMinimalisticLinear } from "@iconify-prerendered/vue-solar"
import { useStorage } from "@vueuse/core"
import { useIrcStore } from "../../stores/irc"
import ListCapabilities from "../shared/server/ListCapabilities.vue"
import { getServerInitials, truncate } from "../../lib/format.ts"
import { computed, ref } from "vue"
import { useConfigStore } from "../../stores/config.ts"
import { useIRCJoinChannel } from "../../composables/useIRCJoinChannel.ts"
import { useUserStore } from "../../stores/user.ts"
// import { useWindowManager } from "../../lib/windows.ts"

const irc = useIrcStore()
const user = useUserStore()
const config = useConfigStore()

config.onShortcut("global:navigation-toggle", () => {
  mini.value = !mini.value
})

const mini = useStorage("orbit-sidebar-state", true)

// Search through servers
// TODO: mini-sidebar search
// TODO: mini-sidebar server peak

const search = ref("")
const serversRaw = computed(() => Array.from(irc.serverData.values()))
const filteredServers = computed(() => serversRaw.value.filter((server) => searchString([server.metadata.name, server.metadata.address], search.value)))

// Join a channel and replace active window
const { join, loading } = useIRCJoinChannel()
// const { replace, focusedWindow } = useWindowManager()

// Replace active window with a channel we've already joined
// async function openChannelWindow(serverId: number, channelId: string) {
//   // FIXME: `f` is not good - location always needs to be set
//   // TODO figure out - if we are not on /wm while replace or any API is called,
//   // should we automatically redirect there? where should that happen?
//   replace(focusedWindow.value?.location ?? "f", {
//     type: "chat",
//     serverId,
//     channelId,
//   })
// }
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
      <RouterLink to="/" class="w-100">
        <DropdownItem>
          <template #icon>
            <IconAddCircleLinear />
          </template>
          Connect
        </DropdownItem>
      </RouterLink>

      <template v-for="server in filteredServers" :key="server.metadata.name">
        <PopoutHover :enter-delay="1000" class="o-sidebar-server-info">
          <template #trigger>
            <DropdownItem class="o-sidebar-server-item">
              <template #icon>
                <Avatar :size="mini ? 'm' : 's'">
                  {{ getServerInitials(server.metadata) }}
                </Avatar>
              </template>
              {{ truncate(server.metadata.name ?? server.metadata.address, 20, "..") }}
            </DropdownItem>
          </template>
          <ListCapabilities :capabilities="server.capabilities" />
        </PopoutHover>

        <div class="o-sidebar-server-channels">
          <DropdownItem :inert="loading" v-for="item in irc.serverChannels.get(server.id)?.joined" @click="join(server.id, item.data.metadata.name)">
            {{ item.data.metadata.name }}
          </DropdownItem>
          <DropdownItem class="lighter" :inert="loading" v-for="item in irc.serverChannels.get(server.id)?.available" @click="join(server.id, item.name)">
            {{ item.name }}
          </DropdownItem>
        </div>
      </template>
    </Flex>

    <template #footer>
      <!-- Minified -->
      <template v-if="user.me.accountName">
        <DropdownItem v-if="mini" x-center expand>
          <template #icon>
            <Avatar url="https://github.com/dolanske.png" size="m"></Avatar>
          </template>
        </DropdownItem>

        <!-- Expanded -->
        <Card class="o-sidebar-user" v-else>
          <Flex y-center gap="xs" expand>
            <!-- <Avatar url="https://github.com/dolanske.png"></Avatar> -->
            <Avatar>{{ user.me.displayName[0].toUpperCase() }}</Avatar>
            <strong class="flex-1">{{ user.me.displayName }}</strong>
            <RouterLink to="/settings">
              <Button square plain>
                <IconSettingsLinear />
              </Button>
            </RouterLink>
          </Flex>
        </Card>
      </template>
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

.o-sidebar-server-info {
  width: 256px;
}

.o-sidebar-server-channels {
  width: -webkit-fill-available;
  width: stretch;
  padding-left: var(--space-m);
  border-left: 1px solid var(--color-border-weak);
  margin-left: calc(var(--space-m) + 2px);
  /* 
  .o-server-channel-available {
    --color-text: var(--color-text-lighter) !important;
  } */
}
</style>
