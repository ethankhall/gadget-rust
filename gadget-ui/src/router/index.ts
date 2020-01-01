import Vue from "vue";
import VueRouter from "vue-router";
import RedirectList from "../views/RedirectList.vue";
import EditRedirect from "../views/EditRedirect.vue";
import CreateRedirect from '../views/CreateRedirect.vue';
import DeleteRedirect from '../views/DeleteRedirect.vue';

Vue.use(VueRouter);

const routes = [
  {
    path: "/",
    name: "home",
    component: RedirectList
  },
  {
    path: "/redirect/edit/:id",
    name: "redirect",
    component: EditRedirect
  },
  {
    path: "/redirect/delete/:id",
    name: "delete-redirect",
    component: DeleteRedirect
  },
  {
    path: "/redirect",
    name: "create-redirect",
    component: CreateRedirect
  },
];

const router = new VueRouter({
  mode: "history",
  base: '/_gadget/ui/',
  routes
});

export default router;
