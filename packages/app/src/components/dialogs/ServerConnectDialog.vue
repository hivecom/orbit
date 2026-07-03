<script setup lang="ts">
import { defineRules, required, useValidation } from "@dolanske/v-valid"
import { Button, Card, Flex, Input } from "@dolanske/vui"
import { reactive, ref } from "vue"

const loading = ref(false)

const form = reactive({
  serverName: "",
})

const emit = defineEmits<{
  success: [data: typeof form]
}>()

const rules = defineRules<typeof form>({
  serverName: [required],
})

const { validate, errors } = useValidation(form, rules, { autoclear: true })

function submit() {
  validate().then(async () => {
    loading.value = true

    await new Promise((resolve) => setTimeout(resolve, 1500))
    // TODO: connect to server
    // Navigate to the server page
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
      <Flex column>
        <Input expand v-model="form.serverName" required :errors="errors.serverName.messages" placeholder="Enter server address" label="Server" />
      </Flex>
    </form>
    <template #footer>
      <!-- <Flex x-end> -->
      <Button expand :loading :inert="loading" @click="submit">Connect</Button>
      <!-- </Flex> -->
    </template>
  </Card>
</template>
