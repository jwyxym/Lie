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
				v-for = '(i, v) in video.list[query.page]'
				:type = "i.url === query.url ? 'primary' : 'default'"
				@click = 'selectEpisode(i)'
			>
				{{ query.offline ? i.name : `第${v + 1}集` }}
			</var-button>
		</var-space>
	</div>
</template>
<script setup lang = 'ts'>
	import { computed, watch, reactive, onMounted, onUnmounted, type ComputedRef } from 'vue';
	import { onBackButtonPress } from '@tauri-apps/api/app';
	import { Snackbar } from '@varlet/ui';

	import { get_ani, get_video, type Anthology } from '@/script/invoke';
	import { useTravel } from '@/script/travel';
	import VideoPlayer from '@/ui/video-player.vue';
	import themes from '@/script/themes';
	import { cachedVideo, readCachedVideo, listDownloads, type Download, type DownloadRequest } from '@/script/downloads';

	const { query, select_episode, leave_video } = useTravel();
	const leaveVideo = () : void => {
		void leave_video();
	};
	const selectEpisode = (episode : { name : string; url : string; }) : void => {
		void select_episode(episode, video.cover);
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
		const url : string = query.value.url;
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

	watch(() : readonly unknown[] => [query.value.url, query.value.back, query.value.name, query.value.offline, query.value.episode, query.value.cover] as const,
		async function ([url, back, name] : readonly unknown[], _previous : readonly unknown[] | undefined, onCleanup : (cleanup : () => void) => void) : Promise<void> {
			let blobUrl : string | undefined;
			const controller : AbortController = new AbortController();
			onCleanup(() : void => {
				controller.abort();
				if (blobUrl) URL.revokeObjectURL(blobUrl);
			});
			if (typeof url !== 'string' || !url)
				return;

			const id : number = ++ request;
			video.src = '';
			video.pageUrl = url;
			video.list = [];
			video.error = '';
			video.local = false;
			video.title = typeof name === 'string' ? name : '';
			video.cover = query.value.cover;
			const offline : boolean = query.value.offline;
			if (offline && query.value.episode) video.title += ` · ${query.value.episode}`;
			try {
				const local : string | undefined = await cachedVideo(url, video.cover);
				if (id !== request) return;
				if (local) {
					if (window.LieAndroid || /Android/i.test(navigator.userAgent)) {
						const blob : Blob = await readCachedVideo(url, controller.signal);
						if (id !== request || disposed) return;
						blobUrl = URL.createObjectURL(blob);
						video.src = blobUrl;
					} else {
						video.src = local;
					}
					video.local = true;
					if (offline) {
						const animeId : string | undefined = new URL(url).pathname.split('/')[2];
						const items : Download[] = await listDownloads();
						if (id !== request) return;
						video.cover ||= items.find((item : Download) : boolean => item.animeId === animeId && !!item.coverUrl)?.coverUrl ?? '';
						video.list = [items.filter((item : Download) : boolean => item.animeId === animeId && item.status === 'completed')
							.map((item : Download) : { name : string; url : string; } => ({ name: item.episodeTitle, url: item.pageUrl }))];
						return;
					}
				}
				if (offline) { video.error = '本地缓存文件不存在，请返回视频缓存页重新缓存'; return; }
			} catch (error : unknown) {
				if (id !== request) return;
				if (offline) { video.error = `读取本地缓存失败：${String(error)}`; return; }
				Snackbar.error(String(error));
			}

			const aniRequest : ReturnType<typeof get_ani> = typeof back === 'string' && back
				? get_ani(back)
				: Promise.resolve(undefined);
			if (!video.local) {
				const src : string = await get_video(url);
				if (id === request)
					video.src = src;
			}

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
