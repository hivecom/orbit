<script setup lang="ts">
import { Button, ButtonGroup, Divider, Dropdown, DropdownItem } from "@dolanske/vui"
import { useWindowManager } from "../../lib/windows"
import { IconHamburgerMenuLinear } from "@iconify-prerendered/vue-solar"

const { windows, split, close, swap, replace } = useWindowManager()
</script>

<template>
  <div class="o-wm">
    <div v-for="(window, location, index) in windows" :class="[`wm-${location}`, 'wm-window']">
      <div class="wm-window-actions">
        <Dropdown>
          <template #trigger="{ toggle }">
            <Button @click="toggle" square plain>
              <IconHamburgerMenuLinear />
            </Button>
          </template>

          <DropdownItem @click="split(location, window)">Split</DropdownItem>
          <DropdownItem @click="close(location)">Close</DropdownItem>

          <template v-for="(w, l) in windows" :key="w?.type">
            <DropdownItem v-if="l !== location" @click="swap(location, l)">
              <!-- TODO: show actual chat title -->
              Swap with {{ l }}
            </DropdownItem>
          </template>
        </Dropdown>
      </div>

      <!-- <h1>{{ window?.type }} | {{ index }}</h1>
      <p v-if="window && window.type !== 'empty'">
        {{ window.type === "chat" ? `Server: ${window.serverId} | Channel: ${window.channelId}` : `Channel: ${window.channelId}` }}
      </p>


      <ButtonGroup :gap="2">
        <Button @click="replace(location, { type: 'chat', serverId: Math.random().toFixed(2), channelId: Math.random().toFixed(2) })">Chat</Button>
        <Button @click="replace(location, { type: 'voice', channelId: Math.random().toFixed(4) })">Voice</Button>
      </ButtonGroup> -->
    </div>
  </div>
</template>

<style scoped>
.o-wm {
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
  gap: var(--space-s);
  width: 100%;
  height: 100%;
  padding: var(--space-s);

  /* Classes which children consume and automatically get positioned */
  .wm-f {
    grid-area: 1 / 1 / 3 / 3;
  }

  .wm-l {
    grid-area: 1 / 1 / 3 / 2;
  }

  .wm-r {
    grid-area: 1 / 2 / 3 / 3;
  }

  .wm-lt {
    grid-area: 1 / 1 / 2 / 2;
  }

  .wm-rt {
    grid-area: 1 / 2 / 2 / 3;
  }

  .wm-lb {
    grid-area: 2 / 1 / 3 / 2;
  }

  .wm-rb {
    grid-area: 2 / 2 / 3 / 3;
  }

  .wm-window {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
    gap: var(--space-l);
    width: 100%;
    height: 100%;
    border-radius: var(--border-radius-m);
    background-color: var(--color-bg-medium);
    position: relative;

    &:has([aria-expanded="true"]),
    &:hover {
      .wm-window-actions {
        visibility: visible;
        pointer-events: all;
      }
    }

    .wm-window-actions {
      visibility: hidden;
      pointer-events: none;
      position: absolute;
      right: 8px;
      top: 8px;
    }
  }
}
</style>
