import { createOrbitApp } from "app"
import { createWebPlatform, initOrbit } from "connector"
import App from "./App.vue"

const servers = await initOrbit()
console.log(servers)
const platform = createWebPlatform()
const app = createOrbitApp(App, platform)

app.mount("#app")
