<template>
	<video ref = 'video' controls/>
</template>

<script setup lang = 'ts'>
	import Hls from 'hls.js';
	import { onMounted, ref } from 'vue';

	const video = ref<HTMLVideoElement | null>(null);
	const props = defineProps<{ src : string; }>();

	onMounted(() => {
			console.log(props.src)
		if (props.src.includes('m3u8') && Hls.isSupported()) {
			const hls = new Hls();
			hls.loadSource(props.src);
			hls.attachMedia(video.value!);

			hls.on(Hls.Events.ERROR, (_, data) => {
				console.error('HLS error:', data)
			})
		} else
			video.value!.src = props.src;
	})
</script>
<style scoped lang = 'scss'>
	video {
		width: 100%;
	}
</style>