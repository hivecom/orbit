<script setup lang="ts">
import { Flex } from "@dolanske/vui"
</script>

<template>
  <Flex y-center x-center column>
    <h2>Orbit</h2>
    <p>coming soon</p>

    <svg width="1024" height="1024" viewBox="0 0 1024 1024" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M499.2 26.0996C521.721 26.0996 543.891 27.602 565.62 30.4941C550.324 52.7044 537.878 77.0214 528.792 102.925C519.016 102.249 509.148 101.9 499.2 101.9C265.639 101.9 76.2999 291.239 76.2998 524.8C76.2998 758.361 265.639 947.7 499.2 947.7C732.761 947.7 922.1 758.361 922.1 524.8C922.1 514.849 921.745 504.979 921.068 495.2C946.972 486.114 971.287 473.669 993.498 458.373C996.391 480.105 997.9 502.277 997.9 524.8C997.9 800.224 774.625 1023.5 499.2 1023.5C223.776 1023.5 0.5 800.224 0.5 524.8C0.50011 249.375 223.776 26.0996 499.2 26.0996ZM819.2 0.5C932.032 0.500099 1023.5 91.9682 1023.5 204.8C1023.5 317.632 932.032 409.1 819.2 409.1C706.368 409.1 614.9 317.632 614.9 204.8C614.9 91.9681 706.368 0.5 819.2 0.5Z"
        stroke="url(#paint0_linear_2320_8)"
      />
      <defs>
        <linearGradient id="paint0_linear_2320_8" x1="976" y1="48" x2="153.6" y2="896" gradientUnits="userSpaceOnUse">
          <stop stop-color="#FC8DC7" />
          <stop offset="1" stop-color="#93EFD8" />
        </linearGradient>
      </defs>
    </svg>
  </Flex>
</template>

<style scoped>
.vui-flex {
  background-color: var(--color-bg);
  height: 100vh;

  p {
    color: var(--color-text-light);
  }

  svg {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    opacity: 0.2;
  }
}
</style>
