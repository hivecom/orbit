<script setup lang="ts">
import { Button, Divider, Input, Switch } from "@dolanske/vui"
import { IconArrowLeftLinear } from "@iconify-prerendered/vue-solar"
import { useConfigStore } from "../../stores/config"

const config = useConfigStore()
</script>

<template>
  <div class="container-m settings-page">
    <div class="settings-title">
      <h2>Settings</h2>
      <Button square plain class="settings-close-button">
        <IconArrowLeftLinear />
      </Button>
    </div>

    <Divider class="my-xl" />

    <section>
      <h3>Appearance</h3>
      <h4>Global</h4>
      <Switch reversed accent label="Zen mode" hint="Greatly simplifies the UI, removing distractions. Can be toggled on/off using the command palette." />
      <h4>Chat</h4>
      <Switch reversed accent label="Colored usernames" hint="Generate a random username color using the username as a seed" />
      <Flex> </Flex>
      <Flex>
        <Switch reversed accent label="Show timestamps" hint="Display timestampts in chat view" />
        <Input label="Timestamp format" v-model="config.options.appearance_chat_timestamps_format" :disabled="!config.options.appearance_chat_timestamps_enabled" />
      </Flex>
    </section>
  </div>
</template>

<style scoped>
.settings-page {
  padding-block: var(--space-s);
  padding-inline: 64px;
  margin: unset;

  :deep(.vui-switch) {
    margin-bottom: var(--space-l);
    .vui-hint {
      display: block;
      max-width: 480px;
    }
  }
}

.settings-title {
  position: relative;

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
  margin-bottom: var(--space-l);
  margin-top: var(--space-xl);
}

h3,
h4 {
  font-weight: var(--font-weight-medium);
}
</style>
