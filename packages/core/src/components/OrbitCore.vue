<script setup lang="ts">
import { Flex } from "@dolanske/vui";
import "../style/index.css";

interface Props {
  environment: 'desktop' | 'mobile' | 'native'
}

const props = defineProps<Props>()

</script>

<template>
  <Flex y-center x-center column>
    <h2>Orbit</h2>
    <p> > > currently in development < < </p>
  </Flex>
</template>

<style>
/* TEMP style */
.vui-flex {
  background-color: var(--color-bg);
  height: 100vh;
}
</style>