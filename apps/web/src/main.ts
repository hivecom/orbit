import { createOrbitApp } from "app"
import { createWebPlatform } from "platform/web"
import App from "./App.vue"

const platform = createWebPlatform()
const app = await createOrbitApp(App, platform)

app.mount("#app")
