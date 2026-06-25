import { createOrbitApp } from "app"
import { createDesktopPlatform } from "platform"
import App from "./App.vue"

const platform = createDesktopPlatform()
const app = createOrbitApp(App, platform)

app.mount("#app")
