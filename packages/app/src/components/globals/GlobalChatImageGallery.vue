<script setup lang="ts">
import { Button, CopyClipboard, Flex, Modal } from "@dolanske/vui"
import { closeMsgModal, isMsgActive, messageModalState } from "../../lib/parse-message"
import { IconArrowLeftLinear, IconArrowRightLinear } from "@iconify-prerendered/vue-solar"
import { computed, ref, watch } from "vue"

const galleryActive = computed(() => !!messageModalState.value?.data.images)
const images = computed(() => (galleryActive.value ? messageModalState.value?.data?.images : []))
const activeIndex = ref(-1)

watch(images, (images: string[]) => {
  if (!images) {
    // When modal is closed, images are set as undefined so we reset index
    activeIndex.value = -1
    return
  }
  const curentIndex = images.indexOf(messageModalState.value?.data.src)
  activeIndex.value = curentIndex
})

function prev() {
  const len = images.value.length
  activeIndex.value = (activeIndex.value - 1 + len) % len
}

function next() {
  activeIndex.value = (activeIndex.value + 1) % images.value.length
}

// FIXME: downloading images won't work unless we proxy this request through our
// own server due to CORS reasons
function download() {}
</script>

<template>
  <Modal
    :card="{ headerAlign: 'start', footerSeparator: true }"
    size="l"
    @close="closeMsgModal()"
    :open="isMsgActive('img')"
    centered
    :style="{
      '--vui-modal-max-width-l': '1024px',
    }"
  >
    <div class="o-global-chat-image-gallery">
      <img :src="images[activeIndex]" alt="Large image" />
    </div>

    <template #footer>
      <Flex x-between y-center>
        <template v-if="images.length > 1">
          <Button square @click="prev()" size="s"><IconArrowLeftLinear /></Button>
          <Button square @click="next()" size="s"><IconArrowRightLinear /></Button>
          <span class="text-xs">{{ activeIndex + 1 }} / {{ images.length }}</span>
        </template>
        <div class="flex-1"></div>
        <CopyClipboard :text="messageModalState!.data.src" confirm="Copied to clipboard">
          <Button plain size="s">Copy link</Button>
        </CopyClipboard>

        <Button plain size="s" disabled @click="download()">Download</Button>
      </Flex>
    </template>
  </Modal>
</template>

<style>
.o-global-chat-image-gallery {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 70vh;

  img {
    display: block;
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: var(--border-radius-s);
  }
}
</style>
