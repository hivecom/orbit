<script setup lang="ts">
import { openMsgModal } from "../../../lib/parse-message"
import { useConfigStore } from "../../../stores/config"

const config = useConfigStore()

const props = defineProps<{
  src: string
}>()

function handle(event: MouseEvent) {
  const galleryId = (event.currentTarget as HTMLButtonElement).dataset.galleryId
  if (!galleryId) {
    return openMsgModal("img", { src: props.src })
  }

  const collection = document.querySelectorAll<HTMLButtonElement>(`[data-gallery-id="${galleryId}"]`)

  if (collection.length < 2) {
    return openMsgModal("img", { src: props.src })
  }

  const images = Array.from(collection).map((el) => (el.firstChild as HTMLImageElement).src)
  openMsgModal("img", { src: props.src, images })
}
</script>

<template>
  <button @click="handle" class="o-chat-img">
    <img :src="props.src" alt="Chat Image" :style="{ 'max-height': config.options.appearance_chat_image_max_height + 'px' }" />
  </button>
</template>
