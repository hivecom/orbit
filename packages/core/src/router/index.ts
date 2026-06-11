import { createMemoryHistory, createRouter } from "vue-router"
import RouteMain from "./views/RouteMain.vue"

const routes = [{ path: "/", component: RouteMain }]

export const router = createRouter({
  history: createMemoryHistory(),
  routes,
})
