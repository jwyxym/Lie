import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import Varlet from '@varlet/ui';
import '@varlet/ui/es/style.mjs';
//@ts-ignore
import 'virtual-icons';

import './script/scale';

createApp(App)
    .use(router)
    .use(Varlet)
    .mount("#app");