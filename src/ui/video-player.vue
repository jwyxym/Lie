<template>
	<video ref = 'video' controls/>
</template>

<script setup lang = 'ts'>
	import Hls from 'hls.js';
	import { onMounted, onUnmounted, ref } from 'vue';

	const video = ref<HTMLVideoElement | null>(null);
	const props = defineProps<{ src : string; }>();
	let hls: Hls | undefined;

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
	}

	onMounted(() => {
		document.addEventListener('fullscreenchange', init_video);
		document.addEventListener('webkitfullscreenchange', init_video);
		if (props.src.includes('m3u8') && Hls.isSupported()) {
			hls = new Hls();
			hls.loadSource(props.src);
			hls.attachMedia(video.value!);

			hls.on(Hls.Events.ERROR, (_, data) => {
				console.error('HLS error:', data)
			})
		} else
			video.value!.src = props.src;
	});

	onUnmounted(() => {
		hls?.destroy();
		document.removeEventListener('fullscreenchange', init_video)
		document.removeEventListener('webkitfullscreenchange', init_video);
	});
</script>
<style scoped lang = 'scss'>
	video {
		width: 100%;
	}
</style>