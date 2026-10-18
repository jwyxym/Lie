import { computed, type ComputedRef } from 'vue';
import { createRouter, createWebHistory, useRoute, useRouter, type LocationQueryRaw, type LocationQueryValue, type RouteLocation, type RouteLocationNormalizedLoaded, type Router } from 'vue-router';
import Home from '@/views/home.vue';
import Animation from '@/views/animation.vue';
import Video from '@/views/video.vue';

export { RouterView as TravelView } from 'vue-router';

export interface TravelQuery {
	home : number;
	day : number;
	search : string;
	series : string;
	url : string;
	back : string;
	name : string;
	page : number;
	cover : string;
	episode : string;
	offline : boolean;
}

export interface VideoDestination {
	url : string;
	back : string;
	name : string;
	page : number;
	cover ?: string;
}

export interface CachedVideoDestination {
	pageUrl : string;
	animeTitle : string;
	episodeTitle : string;
	coverUrl : string;
}

export interface Travel {
	query : ComputedRef<TravelQuery>;
	to_home : (home ?: number, day ?: number, search ?: string) => Promise<void>;
	to_ani : (url : string) => Promise<void>;
	to_video : (destination : VideoDestination) => Promise<void>;
	to_cache : (series ?: string, replace ?: boolean) => Promise<void>;
	to_cached_video : (item : CachedVideoDestination) => Promise<void>;
	select_episode : (episode : { name : string; url : string; }, cover : string) => Promise<void>;
	leave_video : () => Promise<void>;
	back : () => void;
}

export const router : Router = createRouter({
	history : createWebHistory(),
	routes : [
		{
			path : '/downloads', name : 'downloads',
			redirect : (route : RouteLocation) : { name : string; query : LocationQueryRaw; } => ({
				name : 'home', query : { ...route.query, home : '3' }
			})
		},
		{ path : '/', name : 'home', component : Home },
		{ path : '/animation', name : 'animation', component : Animation },
		{ path : '/video', name : 'video', component : Video }
	]
});

const text = (value : LocationQueryValue | LocationQueryValue[] | undefined, fallback : string = '') : string => {
	const first : LocationQueryValue | undefined = Array.isArray(value) ? value[0] : value;
	return typeof first === 'string' ? first : fallback;
};

const index = (value : LocationQueryValue | LocationQueryValue[] | undefined, maximum : number = Number.MAX_SAFE_INTEGER) : number => {
	const parsed : number = Number(text(value, '0'));
	return Number.isSafeInteger(parsed) && parsed >= 0 && parsed <= maximum ? parsed : 0;
};

export const useTravel = () : Travel => {
	const route : RouteLocationNormalizedLoaded = useRoute();
	const navigation : Router = useRouter();
	const query : ComputedRef<TravelQuery> = computed(() : TravelQuery => ({
		home : index(route.query.home, 3), day : index(route.query.day, 6), search : text(route.query.search),
		series : text(route.query.series), url : text(route.query.url), back : text(route.query.back),
		name : text(route.query.name), page : index(route.query.page), cover : text(route.query.cover),
		episode : text(route.query.episode), offline : text(route.query.offline) === '1'
	}));
	// Preserve the selected cached series until the user explicitly returns to the series list.
	const context = () : LocationQueryRaw => ({
		home : query.value.home.toString(), day : query.value.day.toString(), search : query.value.search,
		series : query.value.series || undefined
	});
	const to_home = async (home : number = query.value.home, day : number = query.value.day, search : string = query.value.search) : Promise<void> => {
		await navigation.push({ name : 'home', query : { ...context(), home : home.toString(), day : day.toString(), search } });
	};
	const to_ani = async (url : string) : Promise<void> => {
		await navigation.push({ name : 'animation', query : { ...context(), url } });
	};
	const to_video = async (destination : VideoDestination) : Promise<void> => {
		await navigation.push({ name : 'video', query : {
			...context(), ...destination, page : destination.page.toString(), cover : destination.cover ?? ''
		} });
	};
	const to_cache = async (series : string = query.value.series, replace : boolean = false) : Promise<void> => {
		const target : { name : string; query : LocationQueryRaw; } = {
			name : 'home', query : { ...context(), home : '3', series : series || undefined }
		};
		if (replace) await navigation.replace(target); else await navigation.push(target);
	};
	const to_cached_video = async (item : CachedVideoDestination) : Promise<void> => {
		await navigation.push({ name : 'video', query : {
			...context(), url : item.pageUrl, name : item.animeTitle, episode : item.episodeTitle,
			offline : '1', home : '3', page : '0', cover : item.coverUrl
		} });
	};
	const select_episode = async (episode : { name : string; url : string; }, cover : string) : Promise<void> => {
		if (query.value.offline) {
			await navigation.replace({ name : 'video', query : { ...route.query, url : episode.url, episode : episode.name } });
		} else {
			await to_video({ url : episode.url, back : query.value.back, name : query.value.name, page : query.value.page, cover });
		}
	};
	const leave_video = async () : Promise<void> => {
		if (query.value.offline) {
			const series : string = new URL(query.value.url).pathname.split('/')[2] ?? '';
			await to_cache(series);
		} else if (query.value.back) {
			await to_ani(query.value.back);
		} else {
			await to_home();
		}
	};
	const back = () : void => { navigation.back(); };
	return { query, to_home, to_ani, to_video, to_cache, to_cached_video, select_episode, leave_video, back };
};
