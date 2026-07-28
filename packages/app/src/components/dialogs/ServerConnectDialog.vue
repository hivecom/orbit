<script setup lang="ts">
import { defineRules, minLength, required, useValidation } from "@dolanske/v-valid"
import { Button, Card, Flex, Input } from "@dolanske/vui"
import { reactive, ref } from "vue"
import { useIrcStore } from "../../stores/irc"
import { useRouter } from "vue-router"
import type { Server } from "core-wasm"

const irc = useIrcStore()
const router = useRouter()

const loading = ref(false)

const form = reactive({
  name: "Hivecom",
  url: "wss://irc.hivecom.net:8097",
})

const emit = defineEmits<{
  success: [state: Server]
  error: [error: string]
}>()

const rules = defineRules<typeof form>({
  name: [required, minLength(2)],
  url: [required],
})

const { validate, errors } = useValidation(form, rules, { autoclear: true })

function submit() {
  if (loading.value) {
    return
  }

  validate().then(async () => {
    loading.value = true

    try {
      const { state } = await irc.serverConnect(form.url)

      emit("success", state)
      router.push({ name: "RouteWindowManager" })
    } catch (e) {
      console.log("Error connecting to a server\n", e)
      emit("error", e as string)
    }

    loading.value = false
  })
}
</script>

<template>
  <Card expand separators>
    <Flex class="mb-m" y-center x-between>
      <h3>Connect to server</h3>
      <slot name="stepper"></slot>
    </Flex>
    <form @submit.prevent="submit">
      <Flex column gap="l">
        <Input expand v-model="form.url" required :errors="errors.url.messages" placeholder="Enter server URL..." label="Address" />
        <Input expand v-model="form.name" required :errors="errors.name.messages" placeholder="Enter server name" label="Name" />
      </Flex>
    </form>
    <template #footer>
      <Button expand :loading :inert="loading" @click="submit">Connect</Button>
    </template>
  </Card>
</template>
