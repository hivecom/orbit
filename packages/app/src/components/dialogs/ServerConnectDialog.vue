<script setup lang="ts">
import { defineRules, required, useValidation } from "@dolanske/v-valid"
import { Button, Card, Flex, Input } from "@dolanske/vui"
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
        <Input expand v-model="serverForm.serverName" required :errors="serverErrors.serverName.messages" placeholder="Enter server address..." label="Server" />
      </Flex>
    </form>

    <!-- User sign in -->
    <form @submit.prevent="submitServerConnect" v-else-if="step === Step.SignIn">
      <Flex column>
        <Input expand v-model="userForm.username" required :errors="serverErrors.serverName.messages" placeholder="Enter server address..." label="Server" />
      </Flex>
    </form>
    <template #footer>
      <Flex x-end>
        <Button variant="accent" :loading :inert="loading" @click="submitServerConnect">Connect</Button>
      </Flex>
    </template>
  </Card>
</template>
