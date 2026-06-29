import { createMemoryHistory, createRouter, type RouteRecordRaw, type RouteRecordInfo } from "vue-router"
import RouteMain from "./views/RouteMain.vue"
import RouteChat from "./views/RouteChat.vue"

export const routes = [
  { path: "/", component: RouteMain, name: "RouteMain" },
  { path: "/chat/:serverId/:channelId", component: RouteChat, name: "RouteChat" },
] as const satisfies RouteRecordRaw[]

export const router = createRouter({
  history: createMemoryHistory(),
  routes,
})

// Manually type routes from https://router.vuejs.org/guide/advanced/typed-routes
export interface RouteNamedMap {
  main: RouteRecordInfo<
    // Name
    "main",
    // Path
    "/",
    // Params for 'router.push' or RouterLink's 'to' prop
    Record<never, never>,
    // Normalized param object from 'useRoute'
    Record<never, never>,
    // Union of child route names
    never
  >
  chat: RouteRecordInfo<"chat", "/chat/:serverId/:channelId", { serverId: string; channelId: string }, { serverId: string; channelId: string }, never>
}

// Last, you will need to augment the Vue Router types with this map of routes
declare module "vue-router" {
  interface TypesConfig {
    RouteNamedMap: RouteNamedMap
  }
}
