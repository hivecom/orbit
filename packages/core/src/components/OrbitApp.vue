<script setup lang="ts">
import "../style/index.css";

interface Props {
  environment: 'web' | 'mobile' | 'native'
}

const props = defineProps<Props>()
</script>

<template>
  <div class="orbit">
    <RouterView></RouterView>
  </div>
  <!-- <OrbitApp environment="web" /> -->
</template>

