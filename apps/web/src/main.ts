import { createOrbitApp } from "app"
import { createWebPlatform } from "platform"
import App from "./App.vue"

const platform = createWebPlatform()
const app = createOrbitApp(App, platform)

app.mount("#app")
