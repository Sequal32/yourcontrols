import Vue from "vue";
import VueRouter, { RouteConfig } from "vue-router";
import Loading from "../views/Loading.vue";

Vue.use(VueRouter);

const routes: Array<RouteConfig> = [
  {
    path: "/",
  },
  {
    path: "/loading",
    name: "loading",
    component: Loading
  }
];

const router = new VueRouter({
  routes
});

export default router;
