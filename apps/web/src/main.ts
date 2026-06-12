import { createApp } from "vue"
import { providePlatform, router } from "core"
import { createWebPlatform } from "platform"
import App from "./App.vue"

const app = createApp(App)
app.use(router)
providePlatform(app, createWebPlatform())
app.mount("#app")
