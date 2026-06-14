import { createOrbitApp } from "core"
import { createWebPlatform } from "platform"
import App from "./App.vue"

const app = createOrbitApp(App, createWebPlatform())
app.mount("#app")
