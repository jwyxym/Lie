<template>
	<div class = 'anima no-scrollbar'>
		<var-app-bar
			:title = '
				route.query.name as string
			'
		>
			<template #left>
				<var-button
					round
					text
					@click = 'to_ani(route.query.back as string)'
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
		<var-skeleton :loading = '!video.src'/>
		<video
			v-if = 'video.src'
			:src = 'video.src'
			controls
		/>
		<var-space
			v-if = 'video.src'
		>
			<var-button
				v-for = '(i, v) in video.list[0]'
				:type = "i.url === route.query.url ? 'primary' : 'default'"
				@click = 'to_video(
					i.url,
					route.query.back as string,
					route.query.name as string
				)'
			>
				第{{ v + 1 }}集
			</var-button>
		</var-space>
	</div>
</template>
<script setup lang = 'ts'>
	import { watch, reactive, onUnmounted } from 'vue';
	import { useRoute } from 'vue-router';

	import { get_ani, get_video, type Anthology } from '@/script/invoke';
	import { useTravel } from '@/script/travel';
	import themes from '@/script/themes';

	const { to_video, to_ani } = useTravel();
	const route = useRoute();

	const video = reactive({
		list : [] as Anthology,
		cover : '',
		src : '',
		title : ''
	});

	watch(() => route.query.url, (url) => {
		video.title = route.query.name as string;
		get_ani(route.query.back as string).then(i => {
			if (i)
				video.list = i?.links;
		});
		get_video(url as string).then(i => {
			video.src = i;
		});
	}, { immediate : true });

	onUnmounted(() => {
		URL.revokeObjectURL(video.src);
	})
</script>
<style scoped lang = 'scss'>
	.anima {
		width: 100%;
		height: 100%;
		overflow-y: auto;
		video {
			width: 100%;
		}
		:deep(.var-app-bar) {
			width: 100%;
			.var-app-bar__toolbar {
				width: 100%;
				.var-app-bar__left {
					width: calc(100% - 40px);
				}
			}
		}
	}
</style>