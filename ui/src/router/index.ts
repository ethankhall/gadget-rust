import { createRouter, createWebHistory } from "vue-router";
import RedirectList from "../views/RedirectList.vue";
import EditRedirect from "../views/EditRedirect.vue";
import CreateRedirect from '../views/CreateRedirect.vue';
import DeleteRedirect from '../views/DeleteRedirect.vue';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
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
  ],
});

export default router;