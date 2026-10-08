<script setup lang="ts">
import { Avatar, Flex, Sidebar, Button, Input, ButtonGroup, Tooltip, theme } from "@dolanske/vui"
import { IconAddCircleLinear, IconCloseSquareLinear, IconMagniferLinear, IconSidebarMinimalisticLinear } from "@iconify-prerendered/vue-solar"
import { onClickOutside, onKeyStroke, useStorage } from "@vueuse/core"
import { useIrcStore } from "../../stores/irc"
import { computed, ref, useTemplateRef, watch } from "vue"
import { useConfigStore } from "../../stores/config.ts"
import logo from "../../../public/logo-white-small.svg"
import logoLight from "../../../public/logo-dark-small.svg"
import SidebarServerAccordion from "./SidebarServerAccordion.vue"

// TODO: nested server channels once supported
// TODO: mobile functionality & swipe - gets rid of the mini version and instead completely hides or opens it
// TODO: any missing features I can't think about rn
// TODO: we should track how many messages were in each channel when users last visited them and somehow display if the channel received new messages - or we could track which channels are open and if channel receives new message that is _not_ in the window manager, we show it. Might be simpler that way

const irc = useIrcStore()
const config = useConfigStore()
const mini = useStorage("orbit-sidebar-state", true)

config.onShortcut("global:navigation-toggle", () => {
  mini.value = !mini.value
})

const searchActive = ref(false)
const search = ref("")
const searchRef = useTemplateRef("search")

const sidebarRef = useTemplateRef("sidebar")

// @ts-expect-error Doesn't seem to like receiving vue component
onClickOutside(sidebarRef, () => (searchActive.value = false))
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

const filteredServers = computed(() => irc.filterServersWithChannels(search.value))
</script>

<template>
  <Sidebar :mini ref="sidebar" no-auto-transform variant="plain">
    <Flex column gap="xs" class="mb-m sidebar-header" :y-center="mini">
      <Flex gap="s" :column="mini" y-center>
        <img :src="theme === 'dark' ? logo : logoLight" />
        <ButtonGroup :vertical="mini">
          <Tooltip v-bind="mini ? { placement: 'right' } : {}">
            <Button square @click="mini = !mini" aria-label="Toggle sidebar">
              <IconSidebarMinimalisticLinear />
            </Button>
            <template #tooltip>
              <p>Toggle sidebar</p>
            </template>
          </Tooltip>
          <Tooltip v-bind="mini ? { placement: 'right' } : {}">
            <Button square aria-label="Search" @click="searchActive = true">
              <IconMagniferLinear />
            </Button>
            <template #tooltip>
              <p>Search</p>
            </template>
          </Tooltip>
          <Tooltip v-bind="mini ? { placement: 'right' } : {}">
            <RouterLink to="/">
              <Button square aria-label="Connect">
                <IconAddCircleLinear />
              </Button>
            </RouterLink>
            <template #tooltip>
              <p>Connect to a server</p>
            </template>
          </Tooltip>
          <Tooltip v-bind="mini ? { placement: 'right' } : {}">
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

      <div class="sidebar-search" :class="{ active: searchActive, mini }">
        <Input type="text" placeholder="Search" expand v-model="search" ref="search">
          <template #end>
            <Button square plain size="s" @click="searchActive = false">
              <IconCloseSquareLinear />
            </Button>
          </template>
        </Input>
      </div>
    </Flex>

    <Flex column gap="xs" :y-center="mini">
      <SidebarServerAccordion v-for="server in filteredServers" :key="server.metadata.address" :mini :server />
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

.vui-sidebar-layout .vui-sidebar-outer,
.vui-sidebar {
  transition: none !important;
}

.vui-sidebar {
  border-right: 0;
  user-select: none;

  --vui-sidebar-width-mini: 64px;

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

      &.mini {
        top: 64px;
        left: 100%;
        right: unset;
        bottom: unset;
        width: 192px;
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
    overflow: unset !important;
  }

  .btn-square-override {
    padding-inline: var(--space-xs);
    max-width: unset;
    width: unset;

    .vui-button-slot-default {
      gap: var(--space-xxs);
    }
  }
}
</style>
