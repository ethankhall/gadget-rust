import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
// import 'bootstrap-table/dist/bootstrap-table.js'
// import 'bootstrap-table/dist/bootstrap-table.min.css'

import 'bootstrap/dist/css/bootstrap.css'
import 'bootstrap'
import './assets/custom.scss'

const app = createApp(App);

app.use(router).mount("#app");
