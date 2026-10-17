<template>
	<div class = 'anima'>
		<var-app-bar
			:title = 'video.title'
			title-position = 'left'
		>
			<template #left>
				<var-button
					round
					text
					@click = 'leaveVideo'
				>
					<var-icon
						name = 'chevron-left'
						:size = '24'
					/>
				</var-button>
			</template>
			<template #right>
				<var-icon 
					namespace = 'i'
					:name = 'themes.name'
					:transition = '300'
					@click = 'themes.change()'
				/>
			</template>
		</var-app-bar>
		<var-skeleton :loading = '!video.src && !video.error'/>
		<p v-if = 'video.error' role = 'alert'>{{ video.error }}</p>
		<video-player
			v-if = 'video.src'
			:key = '`${video.pageUrl}:${video.src}`'
			:src = 'video.src'
			:page-url = 'video.pageUrl'
			:cache-request = 'cacheRequest'
			@exit = 'leaveVideo'
		/>
		<br/>
		<var-space
			v-if = 'video.src'
			class = 'no-scrollbar'
		>
			<var-button
				v-for = '(i, v) in video.list[parseInt(route.query.page as string)]'
				:type = "i.url === route.query.url ? 'primary' : 'default'"
				@click = 'selectEpisode(i)'
			>
				{{ route.query.offline === '1' ? i.name : `第${v + 1}集` }}
			</var-button>
		</var-space>
	</div>
</template>
<script setup lang = 'ts'>
	import { computed, watch, reactive, onMounted, onUnmounted, type ComputedRef } from 'vue';
	import { useRoute, useRouter, type Router, type RouteLocationNormalizedLoaded } from 'vue-router';
	import { onBackButtonPress } from '@tauri-apps/api/app';
	import { Snackbar } from '@varlet/ui';

	import { get_ani, get_video, type Anthology } from '@/script/invoke';
	import { useTravel } from '@/script/travel';
	import VideoPlayer from '@/ui/video-player.vue';
	import themes from '@/script/themes';
	import { cachedVideo, listDownloads, type Download, type DownloadRequest } from '@/script/downloads';

	const { to_video, to_ani } = useTravel();
	const route : RouteLocationNormalizedLoaded = useRoute();
	const router : Router = useRouter();
	const leaveVideo = () : void => {
		if (route.query.offline === '1') {
			const series : string | undefined = typeof route.query.url === 'string' ? new URL(route.query.url).pathname.split('/')[2] : undefined;
			void router.push({ name: 'home', query: { home: '3', series } });
			return;
		}
		to_ani(route.query.back as string, route.query.home as string, route.query.day as string, route.query.search as string);
	};
	const selectEpisode = (episode : { name : string; url : string; }) : void => {
		if (route.query.offline === '1') {
			void router.replace({ name: 'video', query: { ...route.query, url: episode.url, episode: episode.name } });
			return;
		}
		to_video(episode.url, route.query.back as string, route.query.name as string,
			parseInt(route.query.page as string), route.query.home as string, route.query.day as string, route.query.search as string, video.cover);
	};

	const video : { list : Anthology; cover : string; src : string; pageUrl : string; title : string; local : boolean; error : string; } = reactive({
		list : [] as Anthology,
		cover : '',
		src : '',
		pageUrl : '',
		title : '',
		local : false,
		error : ''
	});
	let disposed : boolean = false;
	const cacheRequest : ComputedRef<DownloadRequest | undefined> = computed(() : DownloadRequest | undefined => {
		const url : unknown = route.query.url;
		if (typeof url !== 'string' || video.local || !video.title) return undefined;
		const episode : { name : string; url : string; } | undefined = video.list.flat().find((item : { name : string; url : string; }) : boolean => new URL(item.url).pathname === new URL(url).pathname);
		return episode ? { pageUrl: url, animeTitle: video.title, episodeTitle: episode.name, coverUrl: video.cover } : undefined;
	});

	let request : number = 0;

	let listener: Awaited<ReturnType<typeof onBackButtonPress>> | undefined;
	onMounted(async () : Promise<void> => {
		const registered : Awaited<ReturnType<typeof onBackButtonPress>> = await onBackButtonPress(() : void => {
			if (video.src)
				return;
			leaveVideo();
		});
		if (disposed) registered.unregister(); else listener = registered;
	});

	onUnmounted(() : void => {
		disposed = true;
		request++;
		listener?.unregister();
	});

	watch(() : readonly unknown[] => [route.query.url, route.query.back, route.query.name, route.query.offline, route.query.episode, route.query.cover] as const,
		async ([url, back, name] : readonly unknown[]) : Promise<void> => {
			if (typeof url !== 'string')
				return;

			const id : number = ++ request;
			video.src = '';
			video.pageUrl = url;
			video.list = [];
			video.error = '';
			video.local = false;
			video.title = typeof name === 'string' ? name : '';
			video.cover = typeof route.query.cover === 'string' ? route.query.cover : '';
			const offline : boolean = route.query.offline === '1';
			if (offline && typeof route.query.episode === 'string') video.title += ` · ${route.query.episode}`;
			try {
				const local : string | undefined = await cachedVideo(url, video.cover);
				if (id !== request) return;
				if (local) {
					video.src = local;
					video.local = true;
					if (offline) {
						const animeId : string | undefined = new URL(url).pathname.split('/')[2];
						const items : Download[] = await listDownloads();
						if (id !== request) return;
						video.cover ||= items.find((item : Download) : boolean => item.animeId === animeId && !!item.coverUrl)?.coverUrl ?? '';
						video.list = [items.filter((item : Download) : boolean => item.animeId === animeId && item.status === 'completed')
							.map((item : Download) : { name : string; url : string; } => ({ name: item.episodeTitle, url: item.pageUrl }))];
					}
					return;
				}
				if (offline) { video.error = '本地缓存文件不存在，请返回视频缓存页重新缓存'; return; }
			} catch (error : unknown) {
				if (id !== request) return;
				if (offline) { video.error = `读取本地缓存失败：${String(error)}`; return; }
				Snackbar.error(String(error));
			}

			const aniRequest : ReturnType<typeof get_ani> = typeof back === 'string'
				? get_ani(back)
				: Promise.resolve(undefined);
			const src : string = await get_video(url);
			if (id === request)
				video.src = src;

			const ani : Awaited<ReturnType<typeof get_ani>> = await aniRequest;
			if (id === request && ani) {
				video.list = ani.links;
				video.title = ani.name;
				video.cover ||= ani.img;
			}
		}, { immediate : true });
</script>
<style scoped lang = 'scss'>
	.anima {
		width: 100%;
		height: 100%;
		.var-space {
			overflow-y: auto;
			width: 100%;
			height: calc(100% - 300px);
			align-content: flex-start;
		}
	}
</style>
