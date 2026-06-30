import { createRouter, createWebHistory, type RouteRecordInfo } from "vue-router"
import RouteWindowManager from "./views/RouteWindowManager.vue"
import RouteSettings from "./views/RouteSettings.vue"
import RouteMain from "./views/RouteMain.vue"

export const routes = [
  {
    path: "/",
    component: RouteMain,
    name: "RouteMain",
  },
  {
    // Main interaction window manager. It stores all window information as a
    // search parameter. By default, only a fullscreen chat/call/etc view is
    // visible. But user can split this view into 5 different views (on a 2x2 grid).
    name: "RouteWindowManager",
    path: "/wm",
    component: RouteWindowManager,
  },
  // Settings page. Need to figure out if we want each settings section to be a
  // linkable sub-route
  {
    name: "RouteSettings",
    path: "/settings",
    component: RouteSettings,
  },
  // Catch all
  {
    name: "NotFound",
    path: "/:pathMatch(.*)*",
    redirect: "/",
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

// Before resolve guard. Runs before any of the target pages/components are in
// any way loaded. Good place for redirects, auth checks and data fetching
// router.beforeResolve((to, from, next) => {
//   return next(true)
// })

// Manually type routes from https://router.vuejs.org/guide/advanced/typed-routes
export interface RouteNamedMap {
  RouteMain: RouteRecordInfo<
    // Name
    "RouteMain",
    // Path
    "/",
    // Params for 'router.push' or RouterLink's 'to' prop
    Record<never, never>,
    // Normalized param object from 'useRoute'
    Record<never, never>,
    // Union of child route names
    never
  >
  RouteWindowManager: RouteRecordInfo<"RouteWindowManager", "/wm", Record<never, never>, Record<never, never>, never>
  RouteSettings: RouteRecordInfo<"RouteSettings", "/settings", Record<never, never>, Record<never, never>, never>
}

declare module "vue-router" {
  interface TypesConfig {
    RouteNamedMap: RouteNamedMap
  }
}

export { router }
