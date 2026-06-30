<script setup lang="ts">
import { defineRules, required, useValidation } from "@dolanske/v-valid"
import { Button, Card, Flex, Input, Password } from "@dolanske/vui"
import { reactive, ref } from "vue"
import Stepper from "../shared/Stepper.vue"

const Step = {
  Connect: 1,
  SignIn: 2,
  LegacySignIn: 3,
}

const step = ref(Step.Connect)
const loading = ref(false)

/////////////////////////////////////////////////
// 1. Step (connect)
const serverForm = reactive({
  serverName: "",
})

const serverRules = defineRules<typeof serverForm>({
  serverName: [required],
})

const { validate: serverValidate, errors: serverErrors } = useValidation(serverForm, serverRules, { autoclear: true })

function submitServerConnect() {
  serverValidate().then(async () => {
    loading.value = true

    await new Promise((resolve) => setTimeout(resolve, 1500))
    // TODO: connect to server
    // Navigate to the server page
    step.value = 2
    loading.value = false
  })
}

/////////////////////////////////////////////////
// 2. Step (Normal sign in)
const userForm = reactive({
  username: "",
  password: "",
})

const userRules = defineRules<typeof userForm>({
  username: [required],
  password: [required],
})

const { validate: userValidate, errors: userErrors } = useValidation(userForm, userRules, { autoclear: true })

function submitUserSignIn() {
  userValidate().then(async () => {
    loading.value = true

    await new Promise((resolve) => setTimeout(resolve, 1500))

    loading.value = false
  })
}
</script>

<template>
  <Card expand separators>
    <Flex class="mb-m" y-center x-between>
      <h2>Connect</h2>
      <Stepper :steps="2" v-model="step" />
    </Flex>
    <!-- Server connection -->
    <form @submit.prevent="submitServerConnect" v-if="step === Step.Connect">
      <Flex column>
        <Input expand v-model="serverForm.serverName" required :errors="serverErrors.serverName.messages" placeholder="Enter server address" label="Server" />
      </Flex>
    </form>

    <!-- User sign in -->
    <form @submit.prevent="submitUserSignIn" v-else-if="step === Step.SignIn">
      <Flex column>
        <Input expand v-model="userForm.username" required :errors="userErrors.username.messages" placeholder="Enter your username" label="Username" />
        <Password expand v-model="userForm.password" required :errors="userErrors.password.messages" placeholder="********************" label="Password" />
      </Flex>
    </form>
    <template #footer>
      <Flex x-end>
        <Button variant="accent" :loading :inert="loading" @click="submitUserSignIn">Connect</Button>
      </Flex>
    </template>
  </Card>
</template>
