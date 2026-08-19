<script setup lang="ts">
import { Button, Dropdown, DropdownItem } from "@dolanske/vui"
import { useWindowManager, type Window, type WindowLocation } from "../../lib/windows"
import { IconHamburgerMenuLinear } from "@iconify-prerendered/vue-solar"
import { useIrcStore } from "../../stores/irc"
import { EffectScope, effectScope, onBeforeMount, onScopeDispose, useTemplateRef, watch } from "vue"
import { useRouter } from "vue-router"
import WindowEmpty from "../../components/windows/WindowEmpty.vue"
import WindowChat from "../../components/windows/WindowChat.vue"
import { useFocusWithin, whenever } from "@vueuse/core"

const { windows, split, close, swap, focusedWindow, init } = useWindowManager()

const router = useRouter()
const irc = useIrcStore()

const windowRef = useTemplateRef("window")

// Redirect back to main route (username / server setup) if no servers are available
onBeforeMount(() => {
  init()

  if (irc.serverData.size === 0) {
    router.replace({ path: "/" })
  }
})

function getSwapMessage(window: Window | undefined, location: WindowLocation) {
  if (!window) return ""
  switch (window.type) {
    case "chat":
    case "voice":
      return `Swap with ${window.channelId}`
    default:
    case "empty":
      return `Empty window ${location}`
  }
}

// Every time windows update, we register focus checks to determine the
// currently active window. Useful for when user wants to open a new window
// without setting exactly
let focusScope: EffectScope | undefined

watch(
  windowRef,
  () => {
    // Reset previous scope and insert new one
    focusScope?.stop()
    focusScope = effectScope()

    focusScope.run(() => {
      if (!windowRef.value) return

      for (const window of windowRef.value) {
        const { focused } = useFocusWithin(window)

        whenever(focused, () => {
          const location = window.dataset.location as WindowLocation
          const windowObject = windows.value[location]

          if (!windowObject) return

          focusedWindow.value = {
            ...windowObject,
            location,
          }
        })
      }
    })
  },
  {
    flush: "post",
    immediate: true,
  },
)

onScopeDispose(() => focusScope?.stop())
</script>

<template>
  <div class="o-wm">
    <div v-for="(window, location) in windows" :data-location="location" :class="[`wm-${location}`, `wm-${window?.type}`, 'wm-window']" ref="window">
      <div class="wm-window-actions">
        <Dropdown>
          <template #trigger="{ toggle }">
            <Button @click="toggle" square plain>
              <IconHamburgerMenuLinear />
            </Button>
          </template>

          <DropdownItem @click="split(location, window)">Split</DropdownItem>

          <template v-for="(w, l) in windows" :key="w?.type">
            <DropdownItem v-if="l !== location" @click="swap(location, l)">
              {{ getSwapMessage(w, l) }}
            </DropdownItem>
          </template>

          <DropdownItem v-if="Object.keys(windows).length > 1" @click="close(location)">Close</DropdownItem>
        </Dropdown>
      </div>

      <WindowChat v-if="window?.type === 'chat'" v-bind="{ ...window, location }" />
      <WindowEmpty v-else-if="window?.type === 'empty'" />
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
    border: 1px solid var(--color-border-weak);
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

    &.wm-chat {
      justify-content: flex-end;
    }
  }
}
</style>
