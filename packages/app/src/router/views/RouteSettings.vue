<script setup lang="ts">
import { Button, Counter, Divider, Flex, Input, Kbd, Switch } from "@dolanske/vui"
import { IconArrowLeftLinear } from "@iconify-prerendered/vue-solar"
import { useConfigStore } from "../../stores/config"
import { computed } from "vue"

const config = useConfigStore()

const MIN_WIDTH = 25
const MAX_WIDTH = 100

const safeguardedWidth = computed({
  get: () => config.options.appearance_chat_width,
  set: (value) => (config.options.appearance_chat_width = Math.min(MAX_WIDTH, Math.max(value, MIN_WIDTH))),
})
</script>

<template>
  <div class="container-m settings-page">
    <div class="settings-title">
      <h2>Settings</h2>
      <Button square plain class="settings-close-button">
        <IconArrowLeftLinear />
      </Button>
    </div>
    <Divider class="mt-s mb-xl" />
    <section class="settings-section">
      <h3>Appearance</h3>
      <h4>Global</h4>
      <Switch disabled reversed accent label="Zen mode" hint="Greatly simplifies the UI, removing distractions. Can be toggled on/off using the command palette." v-model="config.options.appearance_global_zen_enabled" />
      <h4>Chat</h4>
      <Switch reversed accent label="Colored usernames" hint="Generate a random username color using the username as a seed" v-model="config.options.appearance_chat_colored_usernames" />
      <Switch reversed accent label="Show timestamps" hint="Display timestampts in chat view" v-model="config.options.appearance_chat_timestamps_enabled" />
      <Input label="Timestamp format" v-model="config.options.appearance_chat_timestamps_format" :disabled="!config.options.appearance_chat_timestamps_enabled" />
      <h4>Layout</h4>
      <Flex column :gap="0">
        <label for="chat-width-input" class="vui-label">Chat width</label>
        <p class="vui-hint">Percentual width of the chat compared to its window. On small devices, the width might be automatically adjusted</p>
        <!-- FIXME: doesnt allow typing rn - probably because the automatic clamping immediatel removes it -->
        <Counter id="chat-width-input" :increment-enabled="safeguardedWidth <= MAX_WIDTH" :decrement-enabled="safeguardedWidth >= MIN_WIDTH" type="number" v-model.number="safeguardedWidth" />
      </Flex>
      <Switch reversed accent label="Center chat" hint="If width is other than 100%, the chat will be in the center of the chat window" v-model="config.options.appearance_chat_center_chat" />
      <div class="settings-chat-indicator">
        <div class="width-indicator">
          <div class="width-indicator chat" :class="{ center: config.options.appearance_chat_center_chat }" :style="{ width: config.options.appearance_chat_width + '%' }">
            <span>Chat window</span>
          </div>
        </div>
      </div>
    </section>
    <section class="settings-section">
      <h3>Shortcuts</h3>
      <Flex v-for="value in config.keymap" expand x-between y-center>
        <div>
          <span class="vui-label">{{ value.title }}</span>
          <p class="vui-hint">{{ value.description }}</p>
        </div>
        <Flex gap="xxs">
          <Kbd v-for="key in value.keys.split('+')" :key :keys="key" />
        </Flex>
      </Flex>
    </section>
  </div>
</template>

<style scoped>
.settings-page {
  padding-block: var(--space-s);
  padding-inline: 64px;
  margin: unset;

  :deep(.vui-input) {
    margin-bottom: var(--space-l);
  }

  :deep(.vui-switch) {
    margin-bottom: var(--space-l);

    .vui-hint {
      display: block;
      max-width: 480px;
    }
  }
}

.settings-section {
  margin-bottom: 96px;
  padding-bottom: 96px;
  border-bottom: 1px solid var(--color-border-weak);
}

.settings-chat-indicator {
  display: flex;
  align-items: center;
  background-color: var(--color-bg-medium);
  border-radius: var(--border-radius-l);
  height: 40px;
  padding-inline: var(--space-l);
  corner-shape: squircle;

  .width-indicator {
    width: 100%;
    position: relative;
    border-bottom: 1px solid var(--color-border-strong);
    z-index: 1;
    min-width: 150px;

    span {
      position: absolute;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      padding: 2px var(--space-m);
      background-color: var(--color-bg-medium);
      white-space: nowrap;
      font-size: var(--font-size-xs);
    }

    &.chat {
      border-color: var(--color-accent);
      z-index: 2;

      &.center {
        margin-inline: auto;
      }

      &:before,
      &:after {
        background-color: var(--color-accent);
      }
    }

    &:before,
    &:after {
      content: "";
      position: absolute;
      left: 0;
      top: -8px;
      bottom: -8px;
      width: 1px;
      background-color: var(--color-border-strong);
    }

    &:after {
      left: 100%;
    }
  }
}

.settings-title {
  position: relative;
  padding-bottom: 3px;

  .settings-close-button {
    position: absolute;
    left: -48px;
    top: 50%;
    transform: translateY(-50%);
  }
}

h3 {
  font-size: var(--font-size-xl);
  margin-bottom: var(--space-xl);
}

h4 {
  font-size: var(--font-size-l);
  margin-bottom: var(--space-m);
  margin-top: var(--space-xxl);
}

h3,
h4 {
  font-weight: var(--font-weight-medium);
}
</style>
