import { createMemoryHistory, createRouter, type RouteRecordInfo } from "vue-router"
import RouteWindowManager from "./views/RouteWindowManager.vue"
import RouteSettings from "./views/RouteSettings.vue"
import WindowVoice from "./views/windows/WindowVoice.vue"
import WindowChat from "./views/windows/WindowChat.vue"

export const routes = [
  {
    // Main Window host which allows spawning multiple sub-routes so they can be
    // displayed side by side. By default, it will display only a single
    // sub-route as-s. Like a call, or a chat. But Orbit will always allow
    // tiling multiple interaction windows side by side. To make a window
    // tilable, it needs to be a child-route of WindowTiler
    //
    // Everything else like settings or admin pages cannot be tiled.
    //
    // TODO: still need to figure out how this will work with
    name: "RouteWindowManager",
    path: "/",
    component: RouteWindowManager,
    children: [
      {
        name: "WindowChat",
        path: "/c/:serverId/:channelId",
        component: WindowChat,
      },
      {
        name: "WindowVoice",
        path: "/v/:voiceId",
        component: WindowVoice,
      },
    ],
  },
  {
    name: "RouteSettings",
    path: "/settings",
    component: RouteSettings,
  },
  // Catchall
  {
    name: "NotFound",
    path: "/:pathMatch(.*)*",
    redirect: "/",
  },
]

export const router = createRouter({
  history: createMemoryHistory(),
  routes,
})

// Manually type routes from https://router.vuejs.org/guide/advanced/typed-routes
export interface RouteNamedMap {
  main: RouteRecordInfo<
    // Name
    "RouteWindowManager",
    // Path
    "/",
    // Params for 'router.push' or RouterLink's 'to' prop
    Record<never, never>,
    // Normalized param object from 'useRoute'
    Record<never, never>,
    // Union of child route names
    "WindowChat" | "WindowVoice"
  >
  // Insert routes here
  chat: RouteRecordInfo<"WindowChat", "/c/:serverId/:channelId", { serverId: string; channelId: string }, { serverId: string; channelId: string }, never>
  voice: RouteRecordInfo<"WindowVoice", "/v/:id", { id: string }, { id: string }, never>
}

declare module "vue-router" {
  interface TypesConfig {
    RouteNamedMap: RouteNamedMap
  }
}
