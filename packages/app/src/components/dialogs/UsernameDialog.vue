<script setup lang="ts">
import { defineRules, minLength, required, useValidation } from "@dolanske/v-valid"
import { Button, Card, Flex, Input } from "@dolanske/vui"
import { reactive, ref } from "vue"

const loading = ref(false)

const form = reactive({
  nickname: "",
  displayName: "",
  password: "",
})

const emit = defineEmits<{
  success: [data: typeof form]
}>()

const rules = defineRules<typeof form>({
  nickname: [required, minLength(3)],
  displayName: [required, minLength(3)],
  password: [required],
})

const { validate, errors } = useValidation(form, rules, { autoclear: true })

function submit() {
  validate().then(async () => {
    loading.value = true
    await new Promise((resolve) => setTimeout(resolve, 1500))
    loading.value = false
  })
}
</script>

<template>
  <Card expand separators>
    <Flex class="mb-m" y-center x-between>
      <h2>Username</h2>
      <slot name="stepper"></slot>
    </Flex>
    <form @submit.prevent="submit">
      <Flex column gap="l">
        <Input expand v-model="form.nickname" required placeholder="Enter your username" label="Username" />
        <Input expand v-model="form.displayName" :errors="errors.displayName.messages" required placeholder="Enter your display name" label="Display name" />
      </Flex>
    </form>
    <template #footer>
      <Flex x-end>
        <Button variant="accent" :loading :inert="loading" @click="submit">Create</Button>
      </Flex>
    </template>
  </Card>
</template>
