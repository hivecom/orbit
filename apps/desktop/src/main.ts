import { createOrbitApp } from "app"
import { createDesktopPlatform } from "platform/desktop"
import App from "./App.vue"

const platform = createDesktopPlatform()
const app = await createOrbitApp(App, platform)

app.mount("#app")
