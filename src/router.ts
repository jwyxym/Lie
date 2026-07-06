import { createRouter, createWebHistory } from 'vue-router';
import Home from './views/home.vue';
import Animation from './views/animation.vue';

const router = createRouter({
	history : createWebHistory(),
	routes : [
		{
			path: '/',
			name: 'home',
			component: Home
		},
		{
			path: '/animation',
			name: 'animation',
			component: Animation
		},
	]
});

export default router;