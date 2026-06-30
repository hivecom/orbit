<script setup lang="ts">
import { Button } from "@dolanske/vui"
import { useTiler } from "../../lib/windows"

const { tileset, split, close } = useTiler()
</script>

<template>
  <div class="o-wm">
    <div v-for="(tile, key, index) in tileset" :class="[`wm-${key}`, 'wm-tile']">
      <h1>{{ key }} | {{ index }}</h1>

      <Button @click="split(key, tile)">Split</Button>
      <Button @click="close">Remove</Button>
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

  /* .wm-t {
    grid-area: 1 / 1 / 2 / 3;
  } */

  /* .wm-b {
    grid-area: 2 / 1 / 3 / 3;
  } */

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

  .wm-tile {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    border-radius: var(--border-radius-l);
    background-color: var(--color-bg-raised);
    border: 1px solid var(--color-border);
  }
}
</style>
