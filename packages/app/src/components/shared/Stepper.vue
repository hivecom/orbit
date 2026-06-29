<script setup lang="ts">
// This component is not part of VUI, but if it's used enough, its
// implementation will be cloned there and removed from Orbit

const props = defineProps<{
  /**
   * Total step count
   */
  steps: number
}>()

const activeItemIndex = defineModel({ default: 1 })
</script>

<template>
  <div class="vui-stepper">
    <div v-for="key in steps" :key class="vui-stepper-item" :class="{ active: activeItemIndex === key }" />
  </div>
</template>

<style scoped>
.vui-stepper {
  --vui-stepper-active-color: var(--color-accent);
  --vui-stepper-color: var(--color-text-lightest);
  --vui-stepper-size: 8px;

  display: flex;
  gap: var(--space-xxs);

  .vui-stepper-item {
    height: var(--vui-stepper-size);
    width: var(--vui-stepper-size);
    transition: all var(--transition-slow);
    background-color: var(--color-text-lightest);
    border-radius: var(--border-radius-pill);

    &.active {
      width: 16px;
      background-color: var(--color-accent);
    }
  }
}
</style>
