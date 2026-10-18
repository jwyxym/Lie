import { createApp } from "vue";
import App from "./App.vue";
import { router } from './script/travel';
import Varlet from '@varlet/ui';
import '@varlet/ui/es/style.mjs';
//@ts-ignore
import 'virtual-icons';

import './script/scale';

createApp(App)
    .use(router)
    .use(Varlet)
    .mount("#app");
