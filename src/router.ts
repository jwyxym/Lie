import { createRouter, createWebHistory, type Router } from 'vue-router';
import Home from './views/home.vue';
import Animation from './views/animation.vue';
import Video from './views/video.vue';

const router : Router = createRouter({
	history : createWebHistory(),
	routes : [
		{ path: '/downloads', name: 'downloads', redirect: { name: 'home', query: { home: '3' } } },
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
		{
			path: '/video',
			name: 'video',
			component: Video
		}
	]
});

export default router;
