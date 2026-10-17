<template>
	<div ref = 'container'>
		<VideoPlayer
			ref = 'player'
			:src = 'props.src'
			:download = '!!props.cacheRequest'
			:playback-rates = '[0.5, 1, 1.5, 2, 2.5, 3, 4, 5]'
			@enter-fullscreen = 'in_video'
			@exit-fullscreen = 'out_video'
			@download = 'download'
		/>
	</div>
</template>

<script setup lang = 'ts'>
	import { onBackButtonPress, type OnBackButtonPressPayload } from '@tauri-apps/api/app';
	import { onMounted, onUnmounted, reactive, ref, type Ref } from 'vue';
	import { Snackbar } from '@varlet/ui';
	import { VideoPlayer } from '@jwyxym/video-player';
	import '@jwyxym/video-player/style.css';
	import { cachedVideo, downloadStatus, listDownloads, startDownload, type Download, type DownloadRequest } from '@/script/downloads';
	import { useWatchPosition } from '@/script/watch-position';

	type Player = InstanceType<typeof VideoPlayer>;
	const player : Ref<Player | null> = ref<Player | null>(null);
	const props : { src : string; pageUrl : string; cacheRequest? : DownloadRequest; } = defineProps<{ src : string; pageUrl : string; cacheRequest? : DownloadRequest; }>();
	const container : Ref<HTMLDivElement | null> = ref<HTMLDivElement | null>(null);
	useWatchPosition(container, props.pageUrl);
	const emit : (event : 'exit') => void = defineEmits<{ exit : []; }>();
	const state : { fullscreen : boolean; downloading : boolean; } = reactive({ fullscreen: false, downloading: false });
	let disposed : boolean = false;
	const download = async () : Promise<void> => {
		if (!props.cacheRequest || state.downloading) return;
		const request : DownloadRequest = { ...props.cacheRequest };
		state.downloading = true;
		try {
			const local : string | undefined = await cachedVideo(request.pageUrl, request.coverUrl);
			if (local) {
				if (!disposed) Snackbar.info('本集已缓存，无需重复下载');
				return;
			}
			const items : Download[] = await listDownloads();
			const existing : Download | undefined = items.find((item : Download) : boolean => new URL(item.pageUrl).pathname === new URL(request.pageUrl).pathname);
			if (existing && existing.status !== 'failed') {
				if (!disposed) Snackbar.info(downloadStatus(existing));
				return;
			}
			await startDownload(request);
			if (!disposed) Snackbar.success('已加入缓存任务，可在首页“视频缓存”中管理');
		} catch (error : unknown) { if (!disposed) Snackbar.error(String(error)); }
		finally { state.downloading = false; }
	};

	let listener : Awaited<ReturnType<typeof onBackButtonPress>> | undefined;
	onMounted(async () : Promise<void> => {
		const registered : Awaited<ReturnType<typeof onBackButtonPress>> = await onBackButtonPress((i : OnBackButtonPressPayload) : void => {
			if (state.fullscreen)
				player.value?.exitFullscreen?.();
			else if (i.canGoBack)
				emit('exit');
		});
		if (disposed) registered.unregister(); else listener = registered;
	});

	onUnmounted(() : void => {
		disposed = true;
		listener?.unregister();
	});

	const in_video = () : void => {
		state.fullscreen = true;
		window.LieAndroid?.lockLandscape();
		window.LieAndroid?.hideNavigation();
		window.LieAndroid?.hideStatusBar();
	};

	const out_video = () : void => {
		state.fullscreen = false;
		window.LieAndroid?.lockPortrait();
		window.LieAndroid?.showNavigation();
		window.LieAndroid?.showStatusBar();
	};
</script>
<style scoped lang = 'scss'>
	video {
		width: 100%;
	}
</style>
