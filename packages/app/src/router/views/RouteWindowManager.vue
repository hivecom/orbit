<script setup lang="ts">
import { Button, ButtonGroup, Dropdown, DropdownItem } from "@dolanske/vui"
import { useTiler } from "../../lib/windows"

const { windows, split, close, swap, replace } = useTiler()
</script>

<template>
  <div class="o-wm">
    <div v-for="(window, location, index) in windows" :class="[`wm-${location}`, 'wm-window']">
      <h1>{{ window?.type }} | {{ index }}</h1>
      <p v-if="window && window.type !== 'empty'">
        {{ window.type === "chat" ? `Server: ${window.serverId} | Channel: ${window.channelId}` : `Channel: ${window.channelId}` }}
      </p>

      <ButtonGroup :gap="2">
        <Button @click="split(location, window)">Split</Button>
        <Button @click="close(location)">Remove</Button>
        <Dropdown>
          <template #trigger="{ toggle }">
            <Button @click="toggle">Swap</Button>
          </template>

          <template v-for="(w, l) in windows" :key="w?.type">
            <DropdownItem v-if="l !== location" @click="swap(location, l)">
              With <b>{{ l }}</b>
            </DropdownItem>
          </template>
        </Dropdown>
      </ButtonGroup>

      <ButtonGroup :gap="2">
        <Button @click="replace(location, { type: 'chat', serverId: Math.random().toFixed(2), channelId: Math.random().toFixed(2) })">Chat</Button>
        <Button @click="replace(location, { type: 'voice', channelId: Math.random().toFixed(4) })">Voice</Button>
      </ButtonGroup>
    </div>
  </div>
</template>

<style scoped>
.o-wm {
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
  gap: var(--space-l);
  width: 100%;
  height: 100%;
  padding: var(--space-m);

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
    border-radius: var(--border-radius-l);
    background-color: var(--color-bg-raised);
    border: 1px solid var(--color-border);
  }
}
</style>
