import Vue from "vue";
import VueRouter, { RouteConfig } from "vue-router";
import Loading from "../views/Loading.vue";
import Main from "../views/Main.vue";
import Aircraft from "../views/Aircraft.vue";
import Changelog from "../views/Changelog.vue";
import NetworkTest from "../views/NetworkTest.vue";

Vue.use(VueRouter);

const routes: Array<RouteConfig> = [
  {
    path: "/"
  },
  {
    path: "/loading",
    name: "loading",
    component: Loading
  },
  {
    path: "/main",
    name: "main",
    component: Main
  },
  {
    path: "/aircraft",
    name: "aircraft",
    component: Aircraft
  },
  {
    path: "/changelog",
    name: "changelog",
    component: Changelog
  },
  {
    path: "/networkTest",
    name: "networkTest",
    component: NetworkTest
  }
];

const router = new VueRouter({
  routes
});

export default router;
