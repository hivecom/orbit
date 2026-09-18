<script setup lang="ts">
import { Avatar, Divider, Flex, DropdownItem, Sidebar, Card, Button, PopoutHover, Input, searchString, ButtonGroup, Tooltip } from "@dolanske/vui"
import { IconAddCircleLinear, IconCloseSquareLinear, IconMagniferLinear, IconSettingsLinear, IconSidebarMinimalisticLinear } from "@iconify-prerendered/vue-solar"
import { onClickOutside, onKeyStroke, useMagicKeys, useStorage } from "@vueuse/core"
import { useIrcStore, type IrcChannels, type ServerWithGroupedChannels } from "../../stores/irc"
import { computed, nextTick, onMounted, ref, useTemplateRef, watch } from "vue"
import { useConfigStore } from "../../stores/config.ts"
import logo from "../../../public/logo-white-small.svg"
import SidebarServerAccordion from "./SidebarServerAccordion.vue"
import type { Server } from "core-wasm"
import { toJSON } from "../../lib/helpers.ts"

const irc = useIrcStore()
const config = useConfigStore()

const mini = useStorage("orbit-sidebar-state", true)

config.onShortcut("global:navigation-toggle", () => {
  mini.value = !mini.value
})

const searchActive = ref(false)
const search = ref("")
const searchRef = useTemplateRef("search")

const sidebar = useTemplateRef("sidebar")

// @ts-expect-error Doesn't seem to like receiving vue component
onClickOutside(sidebar, () => (searchActive.value = false))
watch(searchActive, (is) => {
  if (!is) {
    search.value = ""
  } else {
    // Ambiguous but nextTick did not do the trick
    setTimeout(() => {
      if (searchRef.value) {
        searchRef.value.focus()
      }
    }, 50)

    onKeyStroke("Escape", () => (searchActive.value = false))
  }
})

// Get server data and correlated channel data to it as well
const serversRaw = computed(() => {
  const servers: Server[] = Array.from(irc.serverData.values())
  return servers.map((server) => {
    return {
      ...toJSON(server),
      groupedChannels: irc.serverChannels.get(server.id),
    }
  }) as ServerWithGroupedChannels[]
})

const filteredServers = computed(() =>
  serversRaw.value.map((server) => {
    return {
      ...server,
      groupedChannels: {
        joined: server.groupedChannels.joined.filter((channel) => searchString(channel.data.metadata.name, search.value)),
        available: server.groupedChannels.available.filter((channel) => searchString(channel.name, search.value)),
      },
    }
  }),
)

// Join a channel and replace active window
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
  <Sidebar :mini="mini" ref="sidebar">
    <Flex column gap="xs" class="mb-m sidebar-header">
      <Flex gap="s" :column="mini" y-center>
        <img :src="logo" />
        <ButtonGroup :vertical="mini">
          <Tooltip>
            <Button square @click="mini = !mini" aria-label="Toggle sidebar">
              <IconSidebarMinimalisticLinear />
            </Button>
            <template #tooltip>
              <p>Toggle sidebar</p>
            </template>
          </Tooltip>
          <Tooltip>
            <Button square aria-label="Search" @click="searchActive = true">
              <IconMagniferLinear />
            </Button>
            <template #tooltip>
              <p>Search</p>
            </template>
          </Tooltip>
          <Tooltip>
            <RouterLink to="/">
              <Button square aria-label="Connect">
                <IconAddCircleLinear />
              </Button>
            </RouterLink>
            <template #tooltip>
              <p>Connect to a server</p>
            </template>
          </Tooltip>
          <Tooltip>
            <RouterLink to="/settings">
              <Button aria-label="Settings" square>
                <Avatar url="https://github.com/dolanske.png" size="s"></Avatar>
              </Button>
            </RouterLink>
            <template #tooltip>
              <p>Settings</p>
            </template>
          </Tooltip>
        </ButtonGroup>
      </Flex>

      <div class="sidebar-search" :class="{ active: searchActive }">
        <Input type="text" placeholder="Search" expand v-model="search" ref="search">
          <template #end>
            <Button square plain size="s" @click="searchActive = false">
              <IconCloseSquareLinear />
            </Button>
          </template>
        </Input>
      </div>
    </Flex>

    <div style="height: 1px" />
    <Flex column gap="xs">
      <SidebarServerAccordion v-for="server in filteredServers" :key="server.metadata.name" :mini :server />
    </Flex>
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

.vui-sidebar {
  border-right: 0;

  .sidebar-header {
    position: relative;

    .sidebar-search {
      position: absolute;
      inset: 0;
      opacity: 0;
      z-index: -1;
      visibility: hidden;
      transition: all var(--transition);
      transform: scaleX(0);
      padding-right: 2px;

      .vui-input-container {
        --vui-input-background-color: var(--color-button-gray);
        --vui-input-color-border: transparent;
        --border-radius-s: var(--border-radius-m);
        --color-border: transparent;
      }

      &.active {
        opacity: 1;
        z-index: 5;
        visibility: visible;
        transform: scaleX(1);
      }
    }
  }

  .vui-sidebar-content-wrap {
    padding-right: 0 !important;
  }

  .btn-square-override {
    padding-inline: var(--space-xs);
    max-width: unset;
    width: unset;
    /* flex: 1; */

    .vui-button-slot-default {
      gap: var(--space-xxs);
    }
  }
}
</style>
