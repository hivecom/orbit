<script setup lang="ts">
import { defineRules, required, useValidation } from "@dolanske/v-valid"
import { Button, Card, Flex, Input } from "@dolanske/vui"
import { reactive, ref } from "vue"
import { useRouter } from "vue-router"
import Stepper from "../shared/Stepper.vue"

const router = useRouter()
const step = ref(1)

const form = reactive({
  serverName: "",
})

const rules = defineRules<typeof form>({
  serverName: [required],
})

const { validate, errors, reset } = useValidation(form, rules, { autoclear: true })

function submit() {
  validate().then(() => {
    // TODO: connect to server
    // Navigate to the server page
    router.push({
      path: "",
    })
    // router.push({
    //   name: 'RouteChat',
    //   params: {
    //     serverId:
    //   }
    // })
  })
}
</script>

<template>
  <Card expand separators>
    <Flex class="mb-m" y-center x-between>
      <h2>Connect</h2>
      <Stepper :steps="2" v-model="step" />
    </Flex>
    <form @submit.prevent="submit">
      <Flex column>
        <Input expand v-model="form.serverName" required :errors="errors.serverName.messages" placeholder="Enter server address..." label="Server" />
      </Flex>
    </form>
    <template #footer>
      <Flex x-end>
        <Button variant="accent" @click="submit">Connect</Button>
      </Flex>
    </template>
  </Card>
</template>
