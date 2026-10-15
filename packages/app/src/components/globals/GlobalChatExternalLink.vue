<script setup lang="ts">
import { Button, Checkbox, Flex, Modal } from "@dolanske/vui"
import { closeMsgModal, isMsgActive, messageModalState } from "../../lib/parse-message"
import { computed, ref } from "vue"
import { CHAT_URL_TRUSTED_DOMAINS_KEY } from "../../lib/constants"

const domain = computed(() => {
  if (!isMsgActive("link") || !messageModalState.value?.data) return null
  const url = new URL(messageModalState.value?.data)
  return url.hostname
})

const shouldTrust = ref(false)

function goToUrl() {
  if (shouldTrust.value && domain.value) {
    const current = localStorage.getItem(CHAT_URL_TRUSTED_DOMAINS_KEY)
    const list = current ? (JSON.parse(current) as string[]) : []
    list.push(domain.value)
    localStorage.setItem(CHAT_URL_TRUSTED_DOMAINS_KEY, JSON.stringify(list))
  }

  window.open(messageModalState.value!.data, "_blank")
}
</script>

<template>
  <Modal :card="{ headerAlign: 'start', footerSeparator: true }" size="s" @close="closeMsgModal()" :open="isMsgActive('link')" centered>
    <template #header>
      <h3 class="mb-xxs">External link</h3>
      <p class="text-color-light">You are about to leave Orbit via external link. Are you sure the link you're clicking is safe?</p>
    </template>

    <pre>{{ messageModalState?.data }}</pre>
    <Checkbox class="mt-m" v-model="shouldTrust">
      <p>
        Trust <strong>{{ domain }}</strong> from now on
      </p>
    </Checkbox>

    <template #footer>
      <Flex x-end>
        <Button size="s" plain @click="closeMsgModal()">Back</Button>
        <Button size="s" @click="goToUrl">Continue</Button>
      </Flex>
    </template>
  </Modal>
</template>
