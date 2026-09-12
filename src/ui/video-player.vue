<template>
	<VideoPlayer
		ref = 'player'
		:src = 'props.src'
		:download = 'false'
		:playback-rates = '[0.5, 1, 1.5, 2, 2.5, 3, 4, 5]'
		@enter-fullscreen = 'in_video'
		@exit-fullscreen = 'out_video'
	/>
</template>

<script setup lang = 'ts'>
	import { onBackButtonPress } from '@tauri-apps/api/app';
	import { onMounted, onUnmounted, ref } from 'vue';
	import { VideoPlayer } from '@jwyxym/video-player'
	import '@jwyxym/video-player/style.css'
	import { useTravel } from '@/script/travel';

	const tarvel = useTravel()

	const player = ref<InstanceType<typeof VideoPlayer> | null>(null);

	let listener: Awaited<ReturnType<typeof onBackButtonPress>> | undefined;
	let fullscreen = false;
	onMounted(async () => {
		listener = await onBackButtonPress((i) => {
			if (fullscreen)
				player.value?.exitFullscreen?.();
			else if (i.canGoBack)
				tarvel.back();
		});
	});

	onUnmounted(() => {
		listener?.unregister();
	});

	const props = defineProps<{ src : string; }>();

	function in_video () {
		fullscreen = true;
		window.LieAndroid?.lockLandscape();
		window.LieAndroid?.hideNavigation();
		window.LieAndroid?.hideStatusBar();
	};

	function out_video () {
		fullscreen = false;
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