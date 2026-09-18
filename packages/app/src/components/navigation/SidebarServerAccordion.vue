<script setup lang="ts">
import { Avatar, Button, DropdownItem, PopoutHover } from "@dolanske/vui"
import { getServerInitials, truncate } from "../../lib/format"
import { type ServerWithGroupedChannels } from "../../stores/irc"
import { useIRCJoinChannel } from "../../composables/useIRCJoinChannel"
import ListCapabilities from "../shared/server/ListCapabilities.vue"
import { IconAltArrowDownLinear } from "@iconify-prerendered/vue-solar"
import { ref } from "vue"

interface Props {
  server: ServerWithGroupedChannels
  mini: boolean
}

const { server, mini } = defineProps<Props>()
const { join, loading } = useIRCJoinChannel()

const hovering = ref(false)
const open = ref(false)
</script>

<template>
  <PopoutHover :enter-delay="1000" class="o-sidebar-server-info">
    <template #trigger>
      <Button class="o-server-btn" size="s" plain expand @mouseenter="hovering = true" @mouseleave="hovering = false" @click="open = !open">
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
      </Button>
    </template>
    <ListCapabilities :capabilities="server.capabilities" />
  </PopoutHover>

  <div class="o-sidebar-server-channels" v-show="open">
    <DropdownItem :inert="loading" v-for="item in server.groupedChannels.joined" @click="join(server.id, item.data.metadata.name)">
      {{ item.data.metadata.name }}
    </DropdownItem>
    <DropdownItem class="lighter" :inert="loading" v-for="item in server.groupedChannels.available" @click="join(server.id, item.name)">
      {{ item.name }}
    </DropdownItem>
  </div>
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
}
</style>
