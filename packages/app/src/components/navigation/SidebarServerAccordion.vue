<script setup lang="ts">
import { Avatar, Button, DropdownItem, Flex, PopoutHover } from "@dolanske/vui"
import { getServerInitials, truncate } from "../../lib/format"
import { type ServerWithGroupedChannels } from "../../stores/irc"
import { useIRCJoinChannel } from "../../composables/useIRCJoinChannel"
import ListCapabilities from "../shared/server/ListCapabilities.vue"
import { IconAltArrowDownLinear } from "@iconify-prerendered/vue-solar"
import { ref } from "vue"
import { createReusableTemplate, useLocalStorage } from "@vueuse/core"
import { ContextMenu } from "@dolanske/vui"
import { Divider } from "@dolanske/vui"
import { findById, useWindowManager } from "../../lib/windows.ts"

interface Props {
  server: ServerWithGroupedChannels
  mini: boolean
}

const { server, mini } = defineProps<Props>()
const { join, loading } = useIRCJoinChannel()
const { close: closeWindow, windowsCount } = useWindowManager()

const hovering = ref(false)
const open = useLocalStorage(() => `sidebar-${server.id}`, true)

const [DefineChannelList, ReuseChannelList] = createReusableTemplate()
</script>

<template>
  <DefineChannelList>
    <div class="o-sidebar-server-channels" :class="{ mini }">
      <ContextMenu v-for="item in server.groupedChannels.joined" :key="item.metadata.name">
        <DropdownItem :inert="loading" @click="join(server.id, item.metadata.name)">
          {{ item.metadata.name }}
        </DropdownItem>
        <template #menu="{ close: closeMenu }">
          <Flex class="p-xxs" :gap="0" column @click="closeMenu">
            <DropdownItem size="s" @click="join(server.id, item.metadata.name)">Open</DropdownItem>
            <DropdownItem size="s" @click="join(server.id, item.metadata.name, { split: true })">Open in split view</DropdownItem>
            <template v-if="windowsCount > 1 && findById({ channelId: item.metadata.name })">
              <Divider class="my-xxs" />
              <DropdownItem size="s" @click="closeWindow(findById({ channelId: item.metadata.name })!.location)">Close</DropdownItem>
            </template>
            <Divider class="my-xxs" />
            <DropdownItem disabled size="s">Leave</DropdownItem>
          </Flex>
        </template>
      </ContextMenu>
      <ContextMenu v-for="item in server.groupedChannels.available" :key="item.name">
        <DropdownItem class="lighter" :inert="loading" @click="join(server.id, item.name)">
          {{ item.name }}
        </DropdownItem>

        <template #menu="{ close: closeMenu }">
          <Flex class="p-xxs" :gap="0" column @click="closeMenu">
            <DropdownItem size="s" @click="join(server.id, item.name)">Open</DropdownItem>
            <DropdownItem size="s" @click="join(server.id, item.name, { split: true })">Open in split view</DropdownItem>
            <template v-if="windowsCount > 1 && findById({ channelId: item.name })">
              <Divider class="my-xxs" />
              <DropdownItem size="s" @click="closeWindow(findById({ channelId: item.name })!.location)">Close</DropdownItem>
            </template>
          </Flex>
        </template>
      </ContextMenu>
    </div>
  </DefineChannelList>

  <PopoutHover :enter-delay="mini ? 300 : 2000" class="o-sidebar-server-info" v-bind="mini ? { placement: 'right' } : {}">
    <template #trigger>
      <Button class="o-server-btn" :size="mini ? 'l' : 's'" plain expand @mouseenter="hovering = true" @mouseleave="hovering = false" @click="open = !open" :square="mini">
        <template #start>
          <Avatar size="s">
            {{ getServerInitials(server.metadata) }}
          </Avatar>
        </template>
        <template v-if="!mini">
          {{ truncate(server.metadata.name ?? server.metadata.address, 24, "..") }}
          <Avatar size="s" class="icon-avatar" v-show="hovering">
            <IconAltArrowDownLinear class="text-color-light" :style="{ transform: `rotate(${open ? 180 : 0}deg)` }" />
          </Avatar>
        </template>
        <template v-else>
          <Avatar size="m">
            {{ getServerInitials(server.metadata) }}
          </Avatar>
        </template>
      </Button>
    </template>

    <Flex expand column v-if="mini">
      <ReuseChannelList />
    </Flex>
    <ListCapabilities v-else :capabilities="server.capabilities" />
  </PopoutHover>

  <ReuseChannelList v-if="!mini && open" />
</template>

<style>
.o-sidebar-server-info {
  width: 256px;
}

.o-server-btn {
  padding-left: 0 !important;

  .icon-avatar {
    position: absolute;
    right: 0;
    top: 50%;
    transform: translateY(-50%);
    z-index: 5;
  }
}

.o-sidebar-server-item {
  overflow: hidden;
}

.o-sidebar-server-channels {
  width: -webkit-fill-available;
  width: stretch;
  padding-left: calc(var(--space-l) + 2px);
  /* margin-left: calc(var(--space-s) + 2px); */

  &.mini {
    padding: var(--space-s);
  }
}
</style>
