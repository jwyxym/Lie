<template>
	<VideoPlayer
		:src = 'props.src'
		:download = 'false'
		:playback-rates = '[0.5, 1, 1.5, 2, 2.5, 3, 4, 5]'
		@enter-fullscreen = 'in_video'
		@exit-fullscreen = 'out_video'
	/>
</template>

<script setup lang = 'ts'>
	import { VideoPlayer } from '@jwyxym/video-player'
	import '@jwyxym/video-player/style.css'

	const props = defineProps<{ src : string; }>();

	function in_video () {
		window.LieAndroid?.lockLandscape();
		window.LieAndroid?.hideNavigation();
		window.LieAndroid?.hideStatusBar();
	};

	function out_video () {
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