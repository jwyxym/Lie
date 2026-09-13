<template>
	<div class = 'anima'>
		<var-app-bar
			:title = '
				route.query.name as string
			'
		>
			<template #left>
				<var-button
					round
					text
					@click = 'to_ani(
						route.query.back as string,
						route.query.home as string,
						route.query.day as string,
						route.query.search as string
					)'
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
		<video-player
			v-if = 'video.src'
			:key = 'video.src'
			:src = 'video.src'
			@exit = 'to_ani(
				route.query.back as string,
				route.query.home as string,
				route.query.day as string,
				route.query.search as string
			)'
		/>
		<br/>
		<var-space
			v-if = 'video.src'
			class = 'no-scrollbar'
		>
			<var-button
				v-for = '(i, v) in video.list[parseInt(route.query.page as string)]'
				:type = "i.url === route.query.url ? 'primary' : 'default'"
				@click = 'to_video(
					i.url,
					route.query.back as string,
					route.query.name as string,
					parseInt(route.query.page as string),
					route.query.home as string,
					route.query.day as string,
					route.query.search as string
				)'
			>
				第{{ v + 1 }}集
			</var-button>
		</var-space>
	</div>
</template>
<script setup lang = 'ts'>
	import { watch, reactive } from 'vue';
	import { useRoute } from 'vue-router';

	import { get_ani, get_video, type Anthology } from '@/script/invoke';
	import { useTravel } from '@/script/travel';
	import VideoPlayer from '@/ui/video-player.vue';
	import themes from '@/script/themes';

	const { to_video, to_ani } = useTravel();
	const route = useRoute();

	const video = reactive({
		list : [] as Anthology,
		cover : '',
		src : '',
		title : ''
	});

	let request = 0;

	watch(() => [route.query.url, route.query.back, route.query.name] as const,
		async ([url, back, name]) => {
			if (typeof url !== 'string')
				return;

			const id = ++ request;
			video.src = '';
			video.title = typeof name === 'string' ? name : '';

			const aniRequest = typeof back === 'string'
				? get_ani(back)
				: Promise.resolve(undefined);
			const src = await get_video(url);
			// A slower, earlier request must not overwrite the selected episode.
			if (id === request)
				video.src = src;

			const ani = await aniRequest;
			if (id === request && ani)
				video.list = ani.links;
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
