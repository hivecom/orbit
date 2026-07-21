<script setup lang="ts">
import { defineRules, maxLength, minLength, required, useValidation } from "@dolanske/v-valid"
import { Button, Card, Flex, Input } from "@dolanske/vui"
import { reactive, ref } from "vue"
import { useUserStore } from "../../stores/user"

const loading = ref(false)
const user = useUserStore()

const form = reactive({
  // account / realname
  accountName: "orbitske",
  // nickname
  displayName: "orbitske",
  password: "",
})

const emit = defineEmits<{
  success: [data: typeof form]
}>()

const rules = defineRules<typeof form>({
  accountName: [required, minLength(3), maxLength(64)],
  displayName: [required, minLength(3), maxLength(64)],
  // password: [required, minLength(4)],
})

const { validate, errors } = useValidation(form, rules, { autoclear: true })

function submit() {
  validate().then(async () => {
    loading.value = true
    user.signIn(form.accountName, form.displayName, form.password)
    emit("success", form)
    loading.value = false
  })
}
</script>

<template>
  <Card expand separators>
    <Flex class="mb-m" y-center x-between>
      <h3>Create username</h3>
      <slot name="stepper"></slot>
    </Flex>
    <form @submit.prevent="submit">
      <Flex column gap="l">
        <Input expand v-model="form.accountName" required placeholder="Enter your account name" label="Account name" />
        <Input expand v-model="form.displayName" :errors="errors.displayName.messages" required placeholder="Enter your display name" label="Display name" />
        <!-- <Input type="password" expand v-model="form.password" :errors="errors.password.messages" required placeholder="**************" label="Password" /> -->
      </Flex>
    </form>
    <template #footer>
      <!-- <Flex x-end> -->
      <Button expand :loading :inert="loading" @click="submit">Create</Button>
      <!-- </Flex> -->
    </template>
  </Card>
</template>
