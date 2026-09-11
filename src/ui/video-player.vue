<template>
	<VideoPlayer
		:src = 'props.src'
		:download = 'false'
		:playback-rates = '[0.5, 1, 1.5, 2, 2.5, 3, 4, 5]'
		@fullscreenchange = 'init_video'
		@webkitfullscreenchange = 'init_video'
	/>
</template>

<script setup lang = 'ts'>
	import { VideoPlayer } from '@jwyxym/video-player'
	import '@jwyxym/video-player/style.css'

	const props = defineProps<{ src : string; }>();
	let init = 0;

	function init_video () {
		if (init ++ % 2) {
			window.LieAndroid?.lockPortrait();
			window.LieAndroid?.showNavigation();
			window.LieAndroid?.showStatusBar();
		}
		else {
			window.LieAndroid?.lockLandscape();
			window.LieAndroid?.hideNavigation();
			window.LieAndroid?.hideStatusBar();
		}
	};
</script>
<style scoped lang = 'scss'>
	video {
		width: 100%;
	}
</style>